use super::i2c::{bmp085, mcp23017, mcp9808};
use super::RpiApi;
use crate::app::device;
use crate::config::types::Unit;
use crate::error::{Error, Result};
use std::convert::TryInto;

#[derive(Clone, Debug)]
pub struct Device {
    model: device::Type,
    rapi: RpiApi,
    mcp23017_state: mcp23017::Mcp23017State,
    bmp085_state: bmp085::Bmp085State,
}

impl Device {
    pub fn new(model: device::Type, rapi: RpiApi) -> Self {
        // use crate::config::parse::{bool_expr, BoolExpr};
        // let output_scripts = HashMap::new();

        // for (key, val) in outputs.iter() {
        //     let maybe_expression = val.automation_script.map(|a| match bool_expr(a.as_ref()) {
        //         Ok(parsed) => Some(parsed),
        //         Err(e) => {
        //             error!("error parsing automation for output_id {}: {}", key, e);
        //             None
        //         }
        //     });
        //     output_scripts.insert(*key, maybe_expression.flatten());
        // }

        Self {
            model,
            rapi,
            mcp23017_state: mcp23017::Mcp23017State::new(),
            bmp085_state: bmp085::Bmp085State::new(),
        }
    }

    /// Does the device support this input and unit?
    pub fn valid_input(&self, _input_id: i32, _unit: Unit) -> Result<()> {
        Ok(())
    }

    /// Does the device support this output and unit?
    pub fn valid_output(&self, _input_id: i32, _unit: Unit) -> Result<()> {
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        match self.model {
            device::Type::MCP9808(_) => Ok(()),
            device::Type::MCP23017(device::MCP23017 {
                address,
                bank_a,
                bank_b,
            }) => {
                self.mcp23017_state
                    .reset(address.try_into().unwrap(), &self.rapi)
                    .await?;
                self.mcp23017_state
                    .set_pin_directions(
                        address.try_into().unwrap(),
                        mcp23017::Bank::A,
                        &bank_a,
                        &self.rapi,
                    )
                    .await?;
                self.mcp23017_state
                    .set_pin_directions(
                        address.try_into().unwrap(),
                        mcp23017::Bank::B,
                        &bank_b,
                        &self.rapi,
                    )
                    .await?;
                // by writing false, we will update with correct state for all pins after dir change
                self.mcp23017_state
                    .set_pin(
                        address.try_into().unwrap(),
                        mcp23017::Bank::A,
                        mcp23017::Pin::Pin0,
                        false,
                        &self.rapi,
                    )
                    .await?;
                self.mcp23017_state
                    .set_pin(
                        address.try_into().unwrap(),
                        mcp23017::Bank::B,
                        mcp23017::Pin::Pin0,
                        false,
                        &self.rapi,
                    )
                    .await?;
                Ok(())
            }
            device::Type::BMP085(device::BMP085 { address, .. }) => {
                self.bmp085_state
                    .reset(address.try_into().unwrap(), &self.rapi)
                    .await
            }
        }
    }

    pub fn sensor_count(&self) -> Result<u32> {
        Ok(match self.model {
            device::Type::BMP085 { .. } => 2,
            device::Type::MCP9808 { .. } => 1,
            device::Type::MCP23017 { .. } => 0,
        })
    }

    pub fn boolean_count(&self) -> Result<u32> {
        Ok(match self.model {
            device::Type::BMP085 { .. } => 0,
            device::Type::MCP9808 { .. } => 0,
            device::Type::MCP23017 { .. } => 16,
        })
    }

    pub async fn read_boolean(&self, index: i32) -> Result<bool> {
        match self.model {
            device::Type::BMP085 { .. } => Err(Error::OutOfBounds(index as usize)),
            device::Type::MCP9808 { .. } => Err(Error::OutOfBounds(index as usize)),
            device::Type::MCP23017(device::MCP23017 { address, .. }) => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let pin = self
                    .mcp23017_state
                    .get_pin(address.try_into().unwrap(), bank, pin, &self.rapi)
                    .await?;
                Ok(pin)
            }
        }
    }

    pub async fn read_sensor(&self, index: i32) -> Result<(f64, Unit)> {
        match self.model {
            device::Type::BMP085(device::BMP085 { address, mode }) => match index {
                0 => {
                    let v = self
                        .bmp085_state
                        .temperature_in_c(address.try_into().unwrap(), &self.rapi)
                        .await?;
                    Ok((v as f64, Unit::DegC))
                }
                1 => {
                    let v = self
                        .bmp085_state
                        .pressure_kpa(address.try_into().unwrap(), mode, &self.rapi)
                        .await?;
                    Ok((v as f64, Unit::KPa))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            device::Type::MCP9808(device::MCP9808 { address }) => match index {
                0 => {
                    let temp = mcp9808::read_temp(&self.rapi, address.try_into().unwrap()).await?;
                    Ok((temp as f64, Unit::DegC))
                }
                _ => Err(Error::OutOfBounds(index as usize)),
            },
            device::Type::MCP23017(_) => Err(Error::OutOfBounds(index as usize)),
        }
    }

    pub async fn write_boolean(&mut self, index: i32, value: bool) -> Result<()> {
        match self.model {
            device::Type::BMP085(_) => Err(Error::OutOfBounds(index as usize)),
            device::Type::MCP9808(_) => Err(Error::OutOfBounds(index as usize)),
            device::Type::MCP23017(device::MCP23017 {
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
                            address.try_into().unwrap(),
                            bank,
                            pin,
                            *dir_bank.get(index as usize),
                            &self.rapi,
                        )
                        .await?;
                }
                self.mcp23017_state
                    .set_pin(address.try_into().unwrap(), bank, pin, value, &self.rapi)
                    .await
            }
        }
    }
}
