use crate::config;
use crate::i2c::{bmp085, bus::I2cBus, error::Error, mcp23017, mcp9808, Result};
use serde_derive::Serialize;

#[derive(Clone, Serialize, Debug)]
pub enum Status {
    Ok,
}

#[derive(Clone, Debug)]
pub struct Device {
    config: config::Device,
    i2c: I2cBus,
    mcp23017_state: mcp23017::Mcp23017State,
    bmp085_state: bmp085::Bmp085State,
}

impl Device {
    pub fn new(config: &config::Device, i2c: I2cBus) -> Self {
        Device {
            config: config.clone(),
            i2c,
            mcp23017_state: mcp23017::Mcp23017State::new(),
            bmp085_state: bmp085::Bmp085State::new(),
        }
    }

    pub fn status(&self) -> Status {
        Status::Ok
    }

    pub fn config(&self) -> config::Device {
        self.config.clone()
    }

    pub fn set_config(&mut self, config: &config::Device) {
        self.config = config.clone();
    }

    pub fn reset(&mut self) -> Result<()> {
        match &self.config.model {
            config::Type::MCP9808 { .. } => Ok(()),
            config::Type::MCP23017 { address, pins: _ } => {
                self.mcp23017_state.reset(*address, &self.i2c)
            }
            config::Type::BMP085 { address, mode: _ } => {
                self.bmp085_state.reset(*address, &self.i2c)
            }
        }
    }

    pub fn sensor_count(&self) -> usize {
        match &self.config.model {
            config::Type::BMP085 {
                address: _,
                mode: _,
            } => 2,
            config::Type::MCP9808 { address: _ } => 1,
            config::Type::MCP23017 {
                address: _,
                pins: _,
            } => 0,
        }
    }

    pub fn boolean_count(&self) -> usize {
        match self.config.model {
            config::Type::BMP085 {
                address: _,
                mode: _,
            } => 0,
            config::Type::MCP9808 { address: _ } => 0,
            config::Type::MCP23017 {
                address: _,
                pins: _,
            } => 16,
        }
    }

    pub fn read_boolean(&self, index: usize) -> Result<bool> {
        match &self.config.model {
            config::Type::BMP085 { address, mode } => Err(Error::OutOfBounds(index)),
            config::Type::MCP9808 { address } => Err(Error::OutOfBounds(index)),
            config::Type::MCP23017 { address, pins } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                let pin = self
                    .mcp23017_state
                    .get_pin(*address, bank, pin, &self.i2c)?;
                Ok(pin)
            }
        }
    }

    pub fn read_sensor(&self, index: usize) -> Result<(f64, config::Unit)> {
        match &self.config.model {
            config::Type::BMP085 { address, mode } => match index {
                0 => {
                    let v = self.bmp085_state.temperature_in_c(*address, &self.i2c)?;
                    Ok((v as f64, config::Unit::DegC))
                }
                1 => {
                    let v = self.bmp085_state.pressure_kpa(*address, *mode, &self.i2c)?;
                    Ok((v as f64, config::Unit::KPa))
                }
                _ => Err(Error::OutOfBounds(index)),
            },
            config::Type::MCP9808 { address } => match index {
                0 => {
                    let temp = mcp9808::read_temp(&self.i2c, *address)?;
                    Ok((temp as f64, config::Unit::DegC))
                }
                _ => Err(Error::OutOfBounds(index)),
            },
            config::Type::MCP23017 { address, pins } => Err(Error::OutOfBounds(index)),
        }
    }

    pub fn write_boolean(&mut self, index: usize, value: bool) -> Result<()> {
        match &self.config.model {
            config::Type::BMP085 {
                address: _,
                mode: _,
            } => Err(Error::OutOfBounds(index)),
            config::Type::MCP9808 { address: _ } => Err(Error::OutOfBounds(index)),
            config::Type::MCP23017 { address, pins } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                self.mcp23017_state
                    .set_pin(*address, bank, pin, value, &self.i2c)
            }
        }
    }
}
