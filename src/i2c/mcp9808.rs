use crate::i2c::{bus::Address, bus::I2cBus, util::uv2be, Result};
use rand::Rng;

/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf

pub fn read_temp(i2c: &I2cBus, address: Address) -> Result<f32> {
    if cfg!(raspberry_pi) {
        let ts = i2c.read(address, 0x05u8, 2)?;
        let raw = uv2be(&ts);
        let t = raw & 0x0fffu16;
        let sign = if raw & 0x1000u16 == 0x1000u16 {
            -1.0f32
        } else {
            1.0f32
        };
        let temp = (t as f32 / 16f32) * sign;
        Ok(temp)
    } else {
        let rnd = rand::thread_rng().gen_range(-120, 120) as f32 * 100.0f32;

        Ok((rnd / 3.11f32).round() / 100.0)
    }
}
