use crate::i2c::{bus::I2cBus, Result, bus::Address, util::uv2be};


/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf
pub struct Device {
    address: Address,
    i2c: I2cBus,
}

impl Device {
    pub fn new(address: Address, i2c: I2cBus) -> Result<Self>{
       let device = Device {
        address, i2c
       };
       Ok(device)
   }

    pub fn read_temp(&self) -> Result<f32> {
        let ts = self.i2c.read(self.address, 0x05u8, 2)?;
        let raw = uv2be(&ts);
        let t = raw & 0x0fffu16;
        let sign = if raw & 0x1000u16 == 0x1000u16 { -1.0f32 } else { 1.0f32 };
        let temp = (t as f32 / 16f32) * sign;
        Ok(temp)
    }

}

