use crate::app::State;
use crate::config;
use crate::config::boolean::evaluate;
use crate::config::Pin::{Input, Output};
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

    pub fn process(&mut self, state: Arc<Mutex<State>>, _delta: std::time::Duration) {
        match &self.config.model {
            // sensors will have to have their data captured and recorded?
            config::Type::MCP9808 { .. } => {}
            config::Type::BMP085 { .. } => {}
            config::Type::MCP23017 { address, pins } => {
                for (index, pin_state) in pins {
                    if let Ok((bank, pin)) = mcp23017::index_to_bank_pin(*index) {
                        match pin_state {
                            Input { .. } => { /* nothing to do */ }
                            Output {
                                active_low,
                                value,
                                disable_automatic,
                            } => {
                                if disable_automatic
                                    .as_ref()
                                    .map_or(false, |d| evaluate(state, &d))
                                {
                                    if let Some(pv) = value {
                                        let new_pin = evaluate(state, pv);
                                        self.mcp23017_state.set_pin(
                                            *address,
                                            bank,
                                            pin,
                                            *active_low ^ new_pin,
                                            &self.i2c,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
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
            } => 16,
        }
    }

    pub fn switch_count(&self) -> usize {
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

    pub fn read_sensor(&mut self, index: usize) -> Result<(f64, config::Unit)> {
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
            config::Type::MCP23017 { address, pins } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                let active_low = pins.get(&index).map_or(false, |v| v.is_active_low());
                let pin = self
                    .mcp23017_state
                    .get_pin(*address, bank, pin, &self.i2c)?;
                if active_low {
                    Ok((if pin { 0.0f64 } else { 1.0f64 }, config::Unit::Boolean))
                } else {
                    Ok((if pin { 1.0f64 } else { 0.0f64 }, config::Unit::Boolean))
                }
            }
        }
    }

    pub fn write_switch(&mut self, index: usize, value: bool) -> Result<()> {
        match &self.config.model {
            config::Type::BMP085 {
                address: _,
                mode: _,
            } => Err(Error::OutOfBounds(index)),
            config::Type::MCP9808 { address: _ } => Err(Error::OutOfBounds(index)),
            config::Type::MCP23017 { address, pins } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                let active_low = pins.get(&index).map_or(false, |v| v.is_active_low());
                self.mcp23017_state.set_pin(
                    *address,
                    bank,
                    pin,
                    if active_low { !value } else { value },
                    &self.i2c,
                )
            }
        }
    }
}
