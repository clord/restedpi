extern crate chrono;

use crate::config;
use crate::config::value::Unit;
use crate::i2c::{
    bmp085, bus,
    error::Error,
    mcp23017,
    mcp23017::{Bank, Pin},
    mcp9808, Device, Result,
};
use crate::webapp::slugify::slugify;
use chrono::prelude::*;
use std::collections::HashMap;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    devices: HashMap<String, Box<dyn Device>>,
    i2c: bus::I2cBus,
}

unsafe impl Send for State {}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, name: &str, device: Box<dyn Device>) {
        let mut inc: usize = 0;
        while self.devices.contains_key(&slugify(name, inc)) {
            inc += 1;
        }
        self.devices.insert(slugify(name, inc), device);
    }

    pub fn devices(&self) -> &HashMap<String, Box<dyn Device>> {
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
            match m.read_switch(switch) {
                Ok(cur_value) => self.switch_set(name, switch, !cur_value),
                Err(_e) => (),
            };
        }
    }

    pub fn read_sensor(&mut self, name: String, sensor: usize) -> Result<(f64, Unit)> {
        if let Some(m) = self.devices.get(&name) {
            m.read_sensor(sensor)
        } else {
            Err(Error::NonExistant(name))
        }
    }

    pub fn add_devices_from_config(&mut self, configuration: HashMap<String, config::Device>) {
        for (name, config) in configuration.iter() {
            let address = config.address;
            match config.config {
                config::Type::BMP085 { mode } => {
                    let trans_mode = match mode {
                        config::SamplingMode::UltraLowPower => bmp085::SamplingMode::UltraLowPower,
                        config::SamplingMode::Standard => bmp085::SamplingMode::Standard,
                        config::SamplingMode::HighRes => bmp085::SamplingMode::HighRes,
                        config::SamplingMode::UltraHighRes => bmp085::SamplingMode::UltraHighRes,
                    };
                    info!(
                        "Adding BMP085 sensor named '{}' at i2c address {}",
                        name, address
                    );

                    match bmp085::Device::new(name, address, self.i2c.clone(), trans_mode) {
                        Ok(dev) => self.add_device(name, Box::new(dev)),
                        Err(e) => error!("error adding bmp085: {}", e),
                    };
                }
                config::Type::MCP9808 => {
                    info!(
                        "Adding MCP9808 sensor named '{}' at i2c address {}",
                        name, address
                    );
                    match mcp9808::Device::new(name, address, self.i2c.clone()) {
                        Ok(dev) => self.add_device(name, Box::new(dev)),
                        Err(e) => error!("error adding mcp9808: {}", e),
                    };
                }
                config::Type::MCP23017 {
                    ref bank0,
                    ref bank1,
                } => {
                    info!(
                        "Adding MCP23017 switch bank named '{}' at i2c address {}",
                        name, address
                    );
                    match mcp23017::Device::new(name, address, self.i2c.clone()) {
                        Ok(dev) => {
                            for (_bankcfg, _bankname) in [(bank0, Bank::A), (bank1, Bank::B)].iter()
                            {
                                for _pin in [
                                    Pin::Pin0,
                                    Pin::Pin1,
                                    Pin::Pin2,
                                    Pin::Pin3,
                                    Pin::Pin4,
                                    Pin::Pin5,
                                    Pin::Pin6,
                                    Pin::Pin7,
                                ]
                                .iter()
                                {
                                    // if (bankcfg[ordinal(pin)]) {
                                    // set up
                                    // }
                                }
                            }
                            self.add_device(name, Box::new(dev));
                        }
                        Err(e) => error!("error adding mcp23017: {}", e),
                    };
                }
            }
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
