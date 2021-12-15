use super::super::RpiApi;
use super::{util::uv2be, I2cAddress};
use crate::error::Result;

/// MCP 9808
/// High-accuracy temperature Sensor -40°C to +125°C ±0.5°C
/// http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf
pub async fn read_temp(rapi: &RpiApi, address: I2cAddress) -> Result<f32> {
    let ts = rapi.read_i2c(address, 0x05u8, 2).await?;
    let raw = uv2be(&ts);
    let t = raw & 0x0fffu16;
    let neg = (raw & 0x1000u16) == 0x1000u16;
    let sign = if neg { -1.0f32 } else { 1.0f32 };
    let sig_part = if neg {
        256f32 - (t as f32 / 16f32)
    } else {
         (t as f32 / 16f32)
    };
    let temp = sig_part * sign;
    Ok(temp)
}
