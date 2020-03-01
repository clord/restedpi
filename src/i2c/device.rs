use crate::config;
use crate::i2c::{bmp085, bus::I2cBus, error::Error, mcp23017, mcp9808, Result};

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

    pub fn config(&self) -> config::Device {
        self.config.clone()
    }

    pub fn set_config_and_reset(&mut self, config: config::Device) -> Result<()> {
        self.config = config;
        self.reset()
    }

    pub fn reset(&mut self) -> Result<()> {
        match &self.config.model {
            config::Type::MCP9808 => Ok(()),
            config::Type::MCP23017 { bank0: _, bank1: _ } => {
                self.mcp23017_state.reset(self.config.address, &self.i2c)
            }
            config::Type::BMP085 { mode: _ } => {
                self.bmp085_state.reset(self.config.address, &self.i2c)
            }
        }
    }

    pub fn sensor_count(&self) -> usize {
        match &self.config.model {
            config::Type::BMP085 { mode: _ } => 2,
            config::Type::MCP9808 => 1,
            config::Type::MCP23017 { bank0: _, bank1: _ } => 16,
        }
    }

    pub fn switch_count(&self) -> usize {
        match self.config.model {
            config::Type::BMP085 { mode: _ } => 0,
            config::Type::MCP9808 => 0,
            config::Type::MCP23017 { bank0: _, bank1: _ } => 16,
        }
    }

    pub fn read_sensor(&mut self, index: usize) -> Result<(f64, config::Unit)> {
        match &self.config.model {
            config::Type::BMP085 { mode } => match index {
                0 => {
                    let v = self
                        .bmp085_state
                        .temperature_in_c(self.config.address, &self.i2c)?;
                    Ok((v as f64, config::Unit::DegC))
                }
                1 => {
                    let v =
                        self.bmp085_state
                            .pressure_kpa(self.config.address, *mode, &self.i2c)?;
                    Ok((v as f64, config::Unit::KPa))
                }
                _ => Err(Error::OutOfBounds(index)),
            },
            config::Type::MCP9808 => match index {
                0 => {
                    let temp = mcp9808::read_temp(&self.i2c, self.config.address)?;
                    Ok((temp as f64, config::Unit::DegC))
                }
                _ => Err(Error::OutOfBounds(index)),
            },
            config::Type::MCP23017 { bank0, bank1 } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                let active_low = match bank {
                    mcp23017::Bank::A => bank0,
                    mcp23017::Bank::B => bank1,
                }
                .get(&mcp23017::pin_ordinal(pin))
                .map_or(false, |v| v.active_low);
                let pin = self
                    .mcp23017_state
                    .get_pin(self.config.address, bank, pin, &self.i2c)?;
                if active_low {
                    Ok((if pin { 1.0f64 } else { 0.0f64 }, config::Unit::Boolean))
                } else {
                    Ok((if pin { 0.0f64 } else { 1.0f64 }, config::Unit::Boolean))
                }
            }
        }
    }

    pub fn write_switch(&mut self, index: usize, value: bool) -> Result<()> {
        match &self.config.model {
            config::Type::BMP085 { mode: _ } => Err(Error::OutOfBounds(index)),
            config::Type::MCP9808 => Err(Error::OutOfBounds(index)),
            config::Type::MCP23017 { bank0, bank1 } => {
                let (bank, pin) = mcp23017::index_to_bank_pin(index)?;
                let active_low = match bank {
                    mcp23017::Bank::A => bank0,
                    mcp23017::Bank::B => bank1,
                }
                .get(&mcp23017::pin_ordinal(pin))
                .map_or(false, |v| v.active_low);
                if active_low {
                    self.mcp23017_state
                        .set_pin(self.config.address, bank, pin, value, &self.i2c)
                } else {
                    self.mcp23017_state
                        .set_pin(self.config.address, bank, pin, !value, &self.i2c)
                }
            }
        }
    }
}
