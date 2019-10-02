use crate::i2c::util::{iv2be, uv2be};
use crate::i2c::{bus::Address, bus::I2cBus, error::Error, Result, Sensor, Unit};
use std::thread;
use std::time::Duration;

/// How long should we accumulate before returning a result?
#[derive(Clone, Debug)]
pub enum SamplingMode {
    UltraLowPower = 0,
    Standard = 1,
    HighRes = 2,
    UltraHighRes = 3,
}

/// BMP085 command register addresses
enum Control {
    ReadTemp = 0x2E,
    ReadPressure = 0x34,
}

enum Register {
    AC1 = 0xAA, // Calibration
    AC2 = 0xAC, // Calibration
    AC3 = 0xAE, // Calibration
    AC4 = 0xB0, // Calibration
    AC5 = 0xB2, // Calibration
    AC6 = 0xB4, // Calibration
    B1 = 0xB6,  // Calibration
    B2 = 0xB8,  // Calibration
    Mb = 0xBA,  // Calibration
    Mc = 0xBC,  // Calibration
    Md = 0xBE,  // Calibration
    Control = 0xF4,
    Data = 0xF6, // Pressure & Temp
}

#[derive(Clone, Debug)]
struct Coefficients {
    ac1: i16,
    ac2: i16,
    ac3: i16,
    ac4: u16,
    ac5: u16,
    ac6: u16,
    b1: i16,
    b2: i16,
    mb: i16,
    mc: i16,
    md: i16,
}

impl Coefficients {
    pub fn new(address: Address, bus: &I2cBus) -> Result<Self> {
        let ac1 = bus.read(address, Register::AC1 as u8, 2)?;
        let ac2 = bus.read(address, Register::AC2 as u8, 2)?;
        let ac3 = bus.read(address, Register::AC3 as u8, 2)?;
        let ac4 = bus.read(address, Register::AC4 as u8, 2)?;
        let ac5 = bus.read(address, Register::AC5 as u8, 2)?;
        let ac6 = bus.read(address, Register::AC6 as u8, 2)?;
        let b1 = bus.read(address, Register::B1 as u8, 2)?;
        let b2 = bus.read(address, Register::B2 as u8, 2)?;
        let mb = bus.read(address, Register::Mb as u8, 2)?;
        let mc = bus.read(address, Register::Mc as u8, 2)?;
        let md = bus.read(address, Register::Md as u8, 2)?;

        Ok(Coefficients {
            ac1: iv2be(&ac1) as i16,
            ac2: iv2be(&ac2) as i16,
            ac3: iv2be(&ac3) as i16,
            ac4: uv2be(&ac4) as u16,
            ac5: uv2be(&ac5) as u16,
            ac6: uv2be(&ac6) as u16,
            b1: iv2be(&b1) as i16,
            b2: iv2be(&b2) as i16,
            mb: iv2be(&mb) as i16,
            mc: iv2be(&mc) as i16,
            md: iv2be(&md) as i16,
        })
    }
}

/// Represent access to a BMP085 device at 0x77
#[derive(Clone, Debug)]
pub struct Device {
    address: Address,
    accuracy: SamplingMode,
    coefficients: Coefficients,
    i2c: I2cBus,
}

impl Device {
    /// Construct a device with a given sampling mode on a given bus
    pub fn new(address: Address, i2c: I2cBus, accuracy: SamplingMode) -> Result<Self> {
        let coefficients = Coefficients::new(address, &i2c)?;
        let device = Device {
            address,
            accuracy,
            coefficients,
            i2c,
        };
        Ok(device)
    }

    /// Read temperature in degrees c
    pub fn temperature_in_c(&self) -> Result<f32> {
        let (t, _) = self.read_raw_temp()?;
        Ok((t as f32) * 0.1)
    }

    /// Read air pressure in kPa
    pub fn pressure_kpa(&self) -> Result<f32> {
        let (_, b5) = self.read_raw_temp()?;
        let sampling = self.accuracy.clone() as u8;

        let up = self.read_raw_pressure()?;

        let b1: i32 = self.coefficients.b1 as i32;
        let b2: i32 = self.coefficients.b2 as i32;
        let ac1: i32 = self.coefficients.ac1 as i32;
        let ac2: i32 = self.coefficients.ac2 as i32;
        let ac3: i32 = self.coefficients.ac3 as i32;
        let ac4: u32 = self.coefficients.ac4 as u32;

        let b6: i32 = b5 - 4000i32;

        let _t = (b6 as i32).pow(2) >> 12;
        let mut x1: i32 = (b2 * _t) >> 11;
        let mut x2: i32 = (ac2 * b6) >> 11;
        let x3: u32 = (x1 + x2) as u32;
        let b3: i32 = (((ac1 * 4 + (x3 as i32)) << sampling) + 2) / 4;
        x1 = (ac3 * b6) >> 13;
        x2 = (b1 * _t) >> 16;
        let x3: i32 = (x1 + x2 + 2) >> 2;

        let _x3: u32 = (x3 + 32768i32) as u32;
        let b4: u32 = (ac4 * _x3) >> 15;
        let b7: u32 = (up - b3) as u32 * (50000 >> sampling);
        let p = if b7 < 0x80000000 {
            (b7 << 1) / b4
        } else {
            (b7 / b4) << 1
        } as i32;

        x1 = (p >> 8).pow(2);
        x1 = (x1 * 3038) >> 16;
        x2 = (-7357 * (p)) >> 16;

        Ok((((p) + ((x1 + x2 + 3791) >> 4)) as u32) as f32 / 1000f32)
    }

    /// Reads the raw temperature data and associated register
    fn read_raw_temp(&self) -> Result<(i32, i32)> {
        self.i2c.write(
            self.address,
            Register::Control as u8,
            vec![Control::ReadTemp as u8],
        )?;
        thread::sleep(Duration::from_millis(5)); // sleep for 4.5 ms
        let data = self.i2c.read(self.address, Register::Data as u8, 2)?;

        let ut: i32 = iv2be(&data) as i32;
        let ac6: i32 = self.coefficients.ac6 as i32;
        let ac5: i32 = self.coefficients.ac5 as i32;
        let md: i32 = self.coefficients.md as i32;
        let mc: i32 = self.coefficients.mc as i32;

        let _ac5 = ac5 as i64;
        let x1: i32 = ((ut - ac6) as i64 * _ac5 >> 15) as i32; // Note: X>>15 == X/(pow(2,15))
        let x2: i32 = (mc << 11) / (x1 + md); // Note: X<<11 == X<<(pow(2,11))
        let b5: i32 = x1 + x2;
        let t: i32 = (b5 + 8) >> 4;

        Ok((t, b5))
    }

    /// Reads the raw pressure data
    fn read_raw_pressure(&self) -> Result<i32> {
        let pressure_cmd = Control::ReadPressure as u8;
        let sampling = self.accuracy.clone() as u8;
        self.i2c.write(
            self.address,
            Register::Control as u8,
            vec![pressure_cmd + (sampling << 6)],
        )?;

        let duration = match self.accuracy {
            SamplingMode::UltraLowPower => Duration::from_millis(5),
            SamplingMode::Standard => Duration::from_millis(8),
            SamplingMode::HighRes => Duration::from_millis(14),
            SamplingMode::UltraHighRes => Duration::from_millis(26),
        };

        thread::sleep(duration);

        let msbv = self.i2c.read(self.address, Register::Data as u8 + 0, 1)?;
        let lsbv = self.i2c.read(self.address, Register::Data as u8 + 1, 1)?;
        let xlsbv = self.i2c.read(self.address, Register::Data as u8 + 2, 1)?;
        let msb = msbv[0] as u32;
        let lsb = lsbv[0] as u32;
        let xlsb = xlsbv[0] as u32;

        let up: i32 = ((msb << 16) + (lsb << 8) + xlsb >> (8 - sampling)) as i32;

        Ok(up)
    }
}

impl Sensor for Device {
    fn reset(&self) -> Result<()> {
        Ok(())
    }
    fn read_sensor(&self, unit: Unit) -> Result<f64> {
        match unit {
            Unit::DegC => {
                let v = self.temperature_in_c()?;
                Ok(v as f64)
            }
            Unit::KPa => {
                let v = self.pressure_kpa()?;
                Ok(v as f64)
            }
        }
    }
}
