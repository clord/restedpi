extern crate chrono;

use crate::config;
use crate::config::value::Unit;
use crate::i2c::{bus, device::Device, error::Error, Result};
use crate::webapp::slugify::slugify;
use chrono::prelude::*;
use std::collections::HashMap;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    devices: HashMap<String, Device>,
    i2c: bus::I2cBus,
}

// unsafe impl Send for State {}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, name: &str, device: Device) {
        let mut inc: usize = 0;
        while self.devices.contains_key(&slugify(name, inc)) {
            inc += 1;
        }
        self.devices.insert(slugify(name, inc), device);
    }

    pub fn devices(&self) -> &HashMap<String, Device> {
        &self.devices
    }

    pub fn reset(&mut self) {
        for v in self.devices.values_mut() {
            v.reset().expect("reset");
        }
        self.dt = Local::now();
    }

    pub fn set_time(&mut self, t: DateTime<Local>) {
        self.dt = t;
    }

    pub fn current_dt(&self) -> DateTime<Local> {
        self.dt
    }

    pub fn switch_set(&mut self, name: String, switch: usize, value: bool) {
        if let Some(m) = self.devices.get_mut(&name) {
            m.write_switch(switch, value).expect("send write switch");
        }
    }

    pub fn switch_toggle(&mut self, name: String, switch: usize) {
        if let Some(m) = self.devices.get_mut(&name) {
            match m.read_sensor(switch) {
                Ok((cur_value, Unit::Boolean)) => {
                    self.switch_set(name, switch, if cur_value > 0f64 { false } else { true })
                }
                _ => (),
            };
        }
    }

    pub fn read_sensor(&mut self, name: String, sensor: usize) -> Result<(f64, Unit)> {
        if let Some(m) = self.devices.get_mut(&name) {
            m.read_sensor(sensor)
        } else {
            Err(Error::NonExistant(name))
        }
    }

    pub fn add_devices_from_config(&mut self, configuration: HashMap<String, config::Device>) {
        //let result : HashMap<String, Result<()> = Hash::new;
        for (name, config) in configuration.iter() {
            if config.disabled.unwrap_or(false) {
                //result.add(name, Error::Disabled)
                continue;
            }

            let device = Device::new(config, self.i2c.clone());
            let slug = config.slug_name.as_ref().unwrap_or(name);
            self.add_device(&slug, device);

            // let address = config.address;
            // match config.model {
            //     config::Type::BMP085 { mode } => {
            //         info!(
            //             "Adding BMP085 sensor named '{}' at i2c address {}",
            //             name, address
            //         );

            //         match bmp085::Device::new(name, address, self.i2c.clone(), mode) {
            //             Ok(dev) => self.add_device(&slug, dev),
            //             Err(e) => error!("error adding bmp085: {}", e),
            //         };
            //     }
            //     config::Type::MCP9808 => {
            //         info!(
            //             "Adding MCP9808 sensor named '{}' at i2c address {}",
            //             name, address
            //         );
            //         match mcp9808::Device::new(name, address, self.i2c.clone()) {
            //             Ok(dev) => self.add_device(&slug, dev),
            //             Err(e) => error!("error adding mcp9808: {}", e),
            //         };
            //     }
            //     config::Type::MCP23017 {
            //         ref bank0,
            //         ref bank1,
            //     } => {
            //         info!(
            //             "Adding MCP23017 switch bank named '{}' at i2c address {}",
            //             name, address
            //         );
            //         match mcp23017::Device::new(name, address, self.i2c.clone()) {
            //             Ok(dev) => {
            //                 for (_bankcfg, _bankname) in [(bank0, Bank::A), (bank1, Bank::B)].iter()
            //                 {
            //                     for _pin in [
            //                         Pin::Pin0,
            //                         Pin::Pin1,
            //                         Pin::Pin2,
            //                         Pin::Pin3,
            //                         Pin::Pin4,
            //                         Pin::Pin5,
            //                         Pin::Pin6,
            //                         Pin::Pin7,
            //                     ]
            //                     .iter()
            //                     {
            //                         // if (bankcfg[ordinal(pin)]) {
            //                         // set up
            //                         // }
            //                     }
            //                 }
            //                 self.add_device(&slug, dev);
            //             }
            //             Err(e) => error!("error adding mcp23017: {}", e),
            //         };
            //     }
            // }
        }
    }
}

pub fn new() -> State {
    let dt = Local::now();
    let i2c = bus::start();

    State {
        i2c,
        dt,
        devices: HashMap::new(),
    }
}
