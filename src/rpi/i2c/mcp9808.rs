use crate::rpi::{RpiApi, I2cAddress, i2c::util::uv2be};
use crate::error::Result;

/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf

pub fn read_temp(rapi: &RpiApi, address: I2cAddress) -> Result<f32> {
    let ts = rapi.read_i2c(address, 0x05u8, 2)?;
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
