use super::i2c::{bmp085, mcp23017, mcp9808};
use super::RpiApi;
use crate::config;
use crate::error::{Error, Result};
use serde_derive::Serialize;
use std::convert::TryInto;

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

    pub async fn set_config(&mut self, config: &config::Device) -> Result<()> {
        self.config = config.clone();
        self.reset().await
    }

    pub async fn reset(&mut self) -> Result<()> {
        match &self.config.model {
            config::Type::MCP9808(_) => Ok(()),
            config::Type::MCP23017(config::MCP23017Config {
                address,
                bank_a,
                bank_b,
            }) => {
                self.mcp23017_state
                    .reset((*address).try_into().unwrap(), &self.rapi)
                    .await?;
                self.mcp23017_state
                    .set_pin_directions(
                        (*address).try_into().unwrap(),
                        mcp23017::Bank::A,
                        bank_a,
                        &self.rapi,
                    )
                    .await?;
                self.mcp23017_state
                    .set_pin_directions(
                        (*address).try_into().unwrap(),
                        mcp23017::Bank::B,
                        bank_b,
                        &self.rapi,
                    )
                    .await?;
                // by writing false, we will update with correct state for all pins after dir change
                self.mcp23017_state
                    .set_pin(
                        (*address).try_into().unwrap(),
                        mcp23017::Bank::A,
                        mcp23017::Pin::Pin0,
                        false,
                        &self.rapi,
                    )
                    .await?;
                self.mcp23017_state
                    .set_pin(
                        (*address).try_into().unwrap(),
                        mcp23017::Bank::B,
                        mcp23017::Pin::Pin0,
                        false,
                        &self.rapi,
                    )
                    .await?;
                Ok(())
            }
            config::Type::BMP085(config::BMP085Config { address, .. }) => {
                self.bmp085_state
                    .reset((*address).try_into().unwrap(), &self.rapi)
                    .await
            }
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

    pub async fn read_boolean(&self, index: u32) -> Result<bool> {
        match &self.config.model {
            config::Type::BMP085 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP9808 { .. } => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP23017(config::MCP23017Config { address, .. }) => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let pin = self
                    .mcp23017_state
                    .get_pin((*address).try_into().unwrap(), bank, pin, &self.rapi)
                    .await?;
                Ok(pin)
            }
        }
    }

    pub async fn read_sensor(&self, index: u32) -> Result<(f64, config::Unit)> {
        match &self.config.model {
            config::Type::BMP085(config::BMP085Config { address, mode }) => match index {
                0 => {
                    let v = self
                        .bmp085_state
                        .temperature_in_c((*address).try_into().unwrap(), &self.rapi)
                        .await?;
                    Ok((v as f64, config::Unit::DegC))
                }
                1 => {
                    let v = self
                        .bmp085_state
                        .pressure_kpa((*address).try_into().unwrap(), *mode, &self.rapi)
                        .await?;
                    Ok((v as f64, config::Unit::KPa))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            config::Type::MCP9808(config::MCP9808Config { address }) => match index {
                0 => {
                    let temp =
                        mcp9808::read_temp(&self.rapi, (*address).try_into().unwrap()).await?;
                    Ok((temp as f64, config::Unit::DegC))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            config::Type::MCP23017(_) => Err(Error::OutOfBounds(index as usize)),
        }
    }

    pub async fn write_boolean(&mut self, index: u32, value: bool) -> Result<()> {
        match &self.config.model {
            config::Type::BMP085(_) => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP9808(_) => Err(Error::OutOfBounds(index as usize)),
            config::Type::MCP23017(config::MCP23017Config {
                address,
                bank_a,
                bank_b,
            }) => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let old_dir = self.mcp23017_state.get_pin_direction(bank, pin);
                let dir_bank = match bank {
                    mcp23017::Bank::A => bank_a,
                    mcp23017::Bank::B => bank_b,
                };

                if old_dir != *dir_bank.get(index as usize) {
                    self.mcp23017_state
                        .set_pin_direction(
                            (*address).try_into().unwrap(),
                            bank,
                            pin,
                            *dir_bank.get(index as usize),
                            &self.rapi,
                        )
                        .await?;
                }
                self.mcp23017_state
                    .set_pin((*address).try_into().unwrap(), bank, pin, value, &self.rapi)
                    .await
            }
        }
    }
}
