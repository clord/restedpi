use crate::i2c;
use crate::i2c::{
    bus::Address, bus::I2cBus, error::Error, util::uv2be, DeviceType, Direction, Result, Unit,
};

/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf
#[derive(Clone, Debug)]
pub struct Device {
    name: String,
    address: Address,
    i2c: I2cBus,
}

impl Device {
    pub fn new(name: &str, address: Address, i2c: I2cBus) -> Result<Self> {
        let device = Device {
            name: name.to_string(),
            address,
            i2c,
        };
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
    fn reset(&mut self) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Address {
        self.address
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> String {
        "ok".to_string()
    }

    fn description(&self) -> String {
        "MCP 9808".to_string()
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Mcp9808
    }

    fn sensor_count(&self) -> usize {
        return 1;
    }

    fn read_sensor(&self, index: usize) -> Result<(f64, Unit)> {
        match index {
            0 => {
                let v = self.read_temp()?;
                Ok((v as f64, Unit::DegC))
            }
            _ => Err(Error::OutOfBounds(index)),
        }
    }
    fn switch_count(&self) -> usize {
        return 0;
    }
    fn set_direction(&mut self, index: usize, _dir: Direction) -> Result<()> {
        Err(Error::OutOfBounds(index))
    }
    fn switch_direction(&mut self, index: usize) -> Result<Direction> {
        Err(Error::OutOfBounds(index))
    }

    fn write_switch(&mut self, index: usize, _value: bool) -> Result<()> {
        Err(Error::OutOfBounds(index))
    }
    fn read_switch(&mut self, index: usize) -> Result<bool> {
        Err(Error::OutOfBounds(index))
    }
}
