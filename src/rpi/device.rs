use super::i2c::{bmp085, mcp23017, mcp9808};
use super::RpiApi;
use crate::app::device;
use crate::app::dimensioned::Dimensioned;
use crate::config::types::Unit;
use crate::error::{Error, Result};

/// Convert i32 address to u16, returning error if out of valid I2C range
fn to_i2c_addr(address: i32) -> Result<u16> {
    if address < 0 || address > 0x7F {
        Err(Error::OutOfBounds(address as usize))
    } else {
        Ok(address as u16)
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    model: device::Type,
    rapi: RpiApi,
    mcp23017_state: mcp23017::Mcp23017State,
    bmp085_state: bmp085::Bmp085State,
}

impl Device {
    pub fn new(model: device::Type, rapi: RpiApi) -> Self {
        Self {
            model,
            rapi,
            mcp23017_state: mcp23017::Mcp23017State::new(),
            bmp085_state: bmp085::Bmp085State::new(),
        }
    }

    pub fn slots(&self) -> Vec<device::Slot> {
        match self.model {
            device::Type::MCP9808(_) => vec![device::Slot {
                can_input: true,
                can_output: false,
                unit: Unit::DegC,
            }],
            device::Type::BMP085(_) => vec![
                device::Slot {
                    can_input: true,
                    can_output: false,
                    unit: Unit::DegC,
                },
                device::Slot {
                    can_input: true,
                    can_output: false,
                    unit: Unit::KPa,
                },
            ],
            device::Type::MCP23017(device::MCP23017 { bank_a, bank_b, .. }) => {
                let mut result: Vec<device::Slot> = Vec::new();
                for bank in [bank_a, bank_b].iter() {
                    result.push(device::Slot::from_dir(bank.p0));
                    result.push(device::Slot::from_dir(bank.p1));
                    result.push(device::Slot::from_dir(bank.p2));
                    result.push(device::Slot::from_dir(bank.p3));
                    result.push(device::Slot::from_dir(bank.p4));
                    result.push(device::Slot::from_dir(bank.p5));
                    result.push(device::Slot::from_dir(bank.p6));
                    result.push(device::Slot::from_dir(bank.p7));
                }
                result
            }
        }
    }

    pub async fn reset(&mut self) -> Result<()> {
        match self.model {
            device::Type::MCP9808(_) => Ok(()),
            device::Type::MCP23017(device::MCP23017 {
                address,
                bank_a,
                bank_b,
            }) => {
                let addr = to_i2c_addr(address)?;
                self.mcp23017_state.reset(addr, &self.rapi).await?;
                self.mcp23017_state
                    .set_pin_directions(addr, mcp23017::Bank::A, &bank_a, &self.rapi)
                    .await?;
                self.mcp23017_state
                    .set_pin_directions(addr, mcp23017::Bank::B, &bank_b, &self.rapi)
                    .await?;
                // by writing false, we will update with correct state for all pins after dir change
                self.mcp23017_state
                    .set_pin(addr, mcp23017::Bank::A, mcp23017::Pin::Pin0, false, &self.rapi)
                    .await?;
                self.mcp23017_state
                    .set_pin(addr, mcp23017::Bank::B, mcp23017::Pin::Pin0, false, &self.rapi)
                    .await?;
                Ok(())
            }
            device::Type::BMP085(device::BMP085 { address, .. }) => {
                let addr = to_i2c_addr(address)?;
                self.bmp085_state.reset(addr, &self.rapi).await
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
                let addr = to_i2c_addr(address)?;
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let pin_value = self
                    .mcp23017_state
                    .get_pin(addr, bank, pin, &self.rapi)
                    .await?;
                Ok(pin_value)
            }
        }
    }

    pub async fn read_sensor(&self, index: i32) -> Result<Dimensioned> {
        match self.model {
            device::Type::BMP085(device::BMP085 { address, mode }) => {
                let addr = to_i2c_addr(address)?;
                match index {
                    0 => {
                        let v = self
                            .bmp085_state
                            .temperature_in_c(addr, &self.rapi)
                            .await?;
                        Ok(Dimensioned::from_degc(v.into()))
                    }
                    1 => {
                        let v = self
                            .bmp085_state
                            .pressure_kpa(addr, mode, &self.rapi)
                            .await?;
                        Ok(Dimensioned::from_kpa(v.into()))
                    }
                    _ => Err(Error::OutOfBounds(index as usize)),
                }
            }
            device::Type::MCP9808(device::MCP9808 { address }) => {
                let addr = to_i2c_addr(address)?;
                match index {
                    0 => {
                        let temp = mcp9808::read_temp(&self.rapi, addr).await?;
                        Ok(Dimensioned::from_degc(temp.into()))
                    }
                    _ => Err(Error::OutOfBounds(index as usize)),
                }
            }
            device::Type::MCP23017(device::MCP23017 { address, .. }) => {
                let addr = to_i2c_addr(address)?;
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let pin_value = self
                    .mcp23017_state
                    .get_pin(addr, bank, pin, &self.rapi)
                    .await?;
                Ok(Dimensioned::from_bool(pin_value))
            }
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
                let addr = to_i2c_addr(address)?;
                let (bank, pin) = mcp23017::index_to_bank_pin(index as usize);
                let old_dir = self.mcp23017_state.get_pin_direction(bank, pin);
                let dir_bank = match bank {
                    mcp23017::Bank::A => bank_a,
                    mcp23017::Bank::B => bank_b,
                };

                if old_dir != *dir_bank.get(index as usize) {
                    self.mcp23017_state
                        .set_pin_direction(addr, bank, pin, *dir_bank.get(index as usize), &self.rapi)
                        .await?;
                }
                self.mcp23017_state
                    .set_pin(addr, bank, pin, value, &self.rapi)
                    .await
            }
        }
    }
}
