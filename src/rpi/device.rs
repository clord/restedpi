use super::i2c::{bmp085, mcp23017, mcp9808};
use super::RpiApi;
use crate::config;
use crate::error::{Error, Result};
use serde_derive::Serialize;

#[derive(Clone, Serialize, Debug)]
pub enum Status {
    Ok,
}

#[derive(Clone, Debug)]
pub struct Device {
    pub config: config::Device,
    rapi: RpiApi,
    mcp23017_state: mcp23017::Mcp23017State,
    bmp085_state: bmp085::Bmp085State,
}

impl Device {
    pub fn new(config: config::Device, rapi: RpiApi) -> Self {
        Device {
            config,
            rapi,
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

    pub fn set_config(&mut self, config: &config::Device) -> Result<()> {
        self.config = config.clone();
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        match &self.config.model {
            config::Type::MCP9808 { .. } => Ok(()),
            config::Type::MCP23017 {
                address,
                pin_direction,
            } => {
                self.mcp23017_state.reset(*address, &self.rapi)?;
                self.mcp23017_state.set_pin_directions(
                    *address,
                    mcp23017::Bank::A,
                    &pin_direction[0..7],
                    &self.rapi,
                )?;
                self.mcp23017_state.set_pin_directions(
                    *address,
                    mcp23017::Bank::B,
                    &pin_direction[8..15],
                    &self.rapi,
                )?;
                // by writing false, we will update with correct state for all pins after dir change
                self.mcp23017_state.set_pin(
                    *address,
                    mcp23017::Bank::A,
                    mcp23017::Pin::Pin0,
                    false,
                    &self.rapi,
                )?;
                self.mcp23017_state.set_pin(
                    *address,
                    mcp23017::Bank::B,
                    mcp23017::Pin::Pin0,
                    false,
                    &self.rapi,
                )?;
                Ok(())
            }
            config::Type::BMP085 { address, .. } => self.bmp085_state.reset(*address, &self.rapi),
        }
    }

    pub fn sensor_count(&self) -> u32 {
        match &self.config.model {
            config::Type::BMP085 { .. } => 2,
            config::Type::MCP9808 { .. } => 1,
            config::Type::MCP23017 { .. } => 0,
        }
    }

    pub fn boolean_count(&self) -> u32 {
        match self.config.model {
            config::Type::BMP085 { .. } => 0,
            config::Type::MCP9808 { .. } => 0,
            config::Type::MCP23017 { .. } => 16,
        }
    }

    pub fn read_boolean(&self, index: u32) -> Result<bool> {
        match &self.config.model {
            config::Type::BMP085 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP9808 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP23017 { address, .. } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let pin = self
                    .mcp23017_state
                    .get_pin(*address, bank, pin, &self.rapi)?;
                Ok(pin)
            }
        }
    }

    pub fn read_sensor(&self, index: u32) -> Result<(f64, config::Unit)> {
        match &self.config.model {
            config::Type::BMP085 { address, mode } => match index {
                0 => {
                    let v = self.bmp085_state.temperature_in_c(*address, &self.rapi)?;
                    Ok((v as f64, config::Unit::DegC))
                }
                1 => {
                    let v = self
                        .bmp085_state
                        .pressure_kpa(*address, *mode, &self.rapi)?;
                    Ok((v as f64, config::Unit::KPa))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            config::Type::MCP9808 { address } => match index {
                0 => {
                    let temp = mcp9808::read_temp(&self.rapi, *address)?;
                    Ok((temp as f64, config::Unit::DegC))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            config::Type::MCP23017 { .. } => Err(Error::OutOfBounds(index as usize)),
        }
    }

    pub fn write_boolean(&mut self, index: u32, value: bool) -> Result<()> {
        match &self.config.model {
            config::Type::BMP085 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP9808 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP23017 {
                address,
                pin_direction,
            } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let old_dir = self.mcp23017_state.get_pin_direction(bank, pin);
                if old_dir != pin_direction[index as usize] {
                    self.mcp23017_state.set_pin_direction(
                        *address,
                        bank,
                        pin,
                        pin_direction[index as usize],
                        &self.rapi,
                    )?;
                }
                self.mcp23017_state
                    .set_pin(*address, bank, pin, value, &self.rapi)
            }
        }
    }
}
