use crate::i2c;
use crate::i2c::{bus::Address, bus::I2cBus, error::Error, util::uv2be, Result, Unit};
use crate::i2c::{ Sensor, Switch };

/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf
#[derive(Clone, Debug)]
pub struct Device {
    address: Address,
    i2c: I2cBus,
}

impl Device {
    pub fn new(address: Address, i2c: I2cBus) -> Result<Self> {
        let device = Device { address, i2c };
        Ok(device)
    }

    pub fn read_temp(&self) -> Result<f32> {
        let ts = self.i2c.read(self.address, 0x05u8, 2)?;
        let raw = uv2be(&ts);
        let t = raw & 0x0fffu16;
        let sign = if raw & 0x1000u16 == 0x1000u16 {
            -1.0f32
        } else {
            1.0f32
        };
        let temp = (t as f32 / 16f32) * sign;
        Ok(temp)
    }
}

impl i2c::Device for Device {
    fn reset(&self) -> Result<()> {
        Ok(())
    }
    fn address(&self) -> Result<Address> {
        return Ok(self.address)
    }
    fn sensors(&self) -> Vec<Box<dyn Sensor>> {
        return vec![]
    }
    fn switches(&self) -> Vec<Box<dyn Switch>> {
        return vec![]
    }
}

impl i2c::Sensor for Device {
    fn read_sensor(&self, unit: Unit) -> Result<f64> {
        match unit {
            Unit::DegC => {
                let v = self.read_temp()?;
                Ok(v as f64)
            }
            _ => Err(Error::UnsupportedUnit(unit)),
        }
    }

}
