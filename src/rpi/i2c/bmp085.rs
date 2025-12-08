use super::super::RpiApi;
use super::I2cAddress;
use super::util::{iv2be, uv2be};
use crate::app::device::SamplingMode;
use crate::error::Result;
use std::thread;
use std::time::Duration;

/// How long should we accumulate before returning a result?
fn sampling_mode(accuracy: SamplingMode) -> u8 {
    match accuracy {
        SamplingMode::UltraLowPower => 0u8,
        SamplingMode::Standard => 1u8,
        SamplingMode::HighRes => 2u8,
        SamplingMode::UltraHighRes => 3u8,
    }
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

#[derive(Clone, Debug, Default)]
pub struct Bmp085State {
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

impl Bmp085State {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn reset(&mut self, address: I2cAddress, bus: &RpiApi) -> Result<()> {
        // this sensor must be reset to be used...
        let ac1 = bus.read_i2c(address, Register::AC1 as u8, 2).await?;
        let ac2 = bus.read_i2c(address, Register::AC2 as u8, 2).await?;
        let ac3 = bus.read_i2c(address, Register::AC3 as u8, 2).await?;
        let ac4 = bus.read_i2c(address, Register::AC4 as u8, 2).await?;
        let ac5 = bus.read_i2c(address, Register::AC5 as u8, 2).await?;
        let ac6 = bus.read_i2c(address, Register::AC6 as u8, 2).await?;
        let b1 = bus.read_i2c(address, Register::B1 as u8, 2).await?;
        let b2 = bus.read_i2c(address, Register::B2 as u8, 2).await?;
        let mb = bus.read_i2c(address, Register::Mb as u8, 2).await?;
        let mc = bus.read_i2c(address, Register::Mc as u8, 2).await?;
        let md = bus.read_i2c(address, Register::Md as u8, 2).await?;

        // No mutation until all succeed
        self.ac1 = iv2be(&ac1);
        self.ac2 = iv2be(&ac2);
        self.ac3 = iv2be(&ac3);
        self.ac4 = uv2be(&ac4);
        self.ac5 = uv2be(&ac5);
        self.ac6 = uv2be(&ac6);
        self.b1 = iv2be(&b1);
        self.b2 = iv2be(&b2);
        self.mb = iv2be(&mb);
        self.mc = iv2be(&mc);
        self.md = iv2be(&md);
        Ok(())
    }

    /// Read temperature in degrees c
    pub async fn temperature_in_c(&self, address: I2cAddress, rapi: &RpiApi) -> Result<f32> {
        let (t, _) = self.read_raw_temp(address, rapi).await?;
        Ok((t as f32) * 0.1)
    }

    /// Read air pressure in kPa
    pub async fn pressure_kpa(
        &self,
        address: I2cAddress,
        accuracy: SamplingMode,
        rapi: &RpiApi,
    ) -> Result<f32> {
        let (_, b5) = self.read_raw_temp(address, rapi).await?;
        let sampling = sampling_mode(accuracy);

        let up = self.read_raw_pressure(address, accuracy, rapi).await?;

        let b1: i32 = self.b1 as i32;
        let b2: i32 = self.b2 as i32;
        let ac1: i32 = self.ac1 as i32;
        let ac2: i32 = self.ac2 as i32;
        let ac3: i32 = self.ac3 as i32;
        let ac4: u32 = self.ac4 as u32;

        let b6: i32 = b5 - 4000i32;

        let _t = b6.pow(2) >> 12;
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
    async fn read_raw_temp(&self, address: I2cAddress, rapi: &RpiApi) -> Result<(i32, i32)> {
        rapi.write_i2c(
            address,
            Register::Control as u8,
            vec![Control::ReadTemp as u8],
        )
        .await?;
        thread::sleep(Duration::from_millis(5)); // sleep for 4.5 ms
        let data = rapi.read_i2c(address, Register::Data as u8, 2).await?;

        let ut: i32 = iv2be(&data) as i32;
        let ac6: i32 = self.ac6 as i32;
        let ac5: i32 = self.ac5 as i32;
        let md: i32 = self.md as i32;
        let mc: i32 = self.mc as i32;

        let _ac5 = ac5 as i64;
        let x1: i32 = (((ut - ac6) as i64 * _ac5) >> 15) as i32;
        if (x1 + md) == 0 {
            return Err(crate::error::Error::DeviceReadError(
                "BMP085 will divide by zero".to_string(),
            ));
        }
        let x2: i32 = (mc << 11) / (x1 + md);
        let b5: i32 = x1 + x2;
        let t: i32 = (b5 + 8) >> 4;

        Ok((t, b5))
    }

    /// Reads the raw pressure data
    async fn read_raw_pressure(
        &self,
        address: I2cAddress,
        accuracy: SamplingMode,
        rapi: &RpiApi,
    ) -> Result<i32> {
        let pressure_cmd = Control::ReadPressure as u8;
        let sampling = sampling_mode(accuracy);
        rapi.write_i2c(
            address,
            Register::Control as u8,
            vec![pressure_cmd + (sampling << 6)],
        )
        .await?;

        let duration = match accuracy {
            SamplingMode::UltraLowPower => Duration::from_millis(5),
            SamplingMode::Standard => Duration::from_millis(8),
            SamplingMode::HighRes => Duration::from_millis(14),
            SamplingMode::UltraHighRes => Duration::from_millis(26),
        };

        thread::sleep(duration);

        let msbv = rapi.read_i2c(address, Register::Data as u8, 1).await?;
        let lsbv = rapi.read_i2c(address, Register::Data as u8 + 1, 1).await?;
        let xlsbv = rapi.read_i2c(address, Register::Data as u8 + 2, 1).await?;
        let msb = msbv[0] as u32;
        let lsb = lsbv[0] as u32;
        let xlsb = xlsbv[0] as u32;

        let up: i32 = (((msb << 16) + (lsb << 8) + xlsb) >> (8 - sampling)) as i32;

        Ok(up)
    }
}
