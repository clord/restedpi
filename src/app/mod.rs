extern crate chrono;

use crate::config;
use crate::config::value::Unit;
use crate::i2c::{
    bmp085, bus,
    bus::{Address, I2cBus},
    error::Error,
    mcp23017,
    mcp23017::{Bank, Pin},
    mcp9808, Device, Result,
};
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    devices: HashMap<String, Box<dyn Device>>,
    i2c: I2cBus,
}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, name: String, sensor: Box<dyn Device>) {
        self.devices.insert(name, sensor);
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

    pub fn current_time(&self) {
        self.dt
    }

    pub fn switch_set(&mut self, name: String, switch: usize, pin: usize, value: bool) {
        if let Some(m) = self.devices.get_mut(&name) {
            m[switch]
                .write_switch(pin, value)
                .expect("send write switch");
        }
    }

    pub fn switch_toggle(&mut self, name: String, switch: usize, pin: usize) {
        if let Some(m) = self.devices.get_mut(&name) {
            match m[switch].read_switch(pin) {
                Ok(cur_value) => self.switch_set(name, pin, switch, pin, !cur_value),
                Err(_e) => (),
            };
        }
    }

    pub fn read_sensor(&mut self, name: String, sensor: usize, unit: Unit) {
        if let Some(m) = self.devices.get(&name) {
            m[sensor].read_sensor(unit)
        } else {
            Err(Error::NonExistant(name))
        }
    }

    pub fn add_switches_from_config(&mut self, switch_config: HashMap<String, config::Switch>) {
        let switches = HashMap::new();
        for (name, config) in switch_config.iter() {
            match config.device {
                config::SwitchType::MCP23017 {
                    address,
                    ref bank0,
                    ref bank1,
                } => {
                    info!(
                        "Adding MCP23017 switch bank named '{}' at i2c address {}",
                        name, address
                    );
                    match (mcp23017::Device::new(address, self.i2c.clone())) {
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
                            self.add_device(name.to_string(), Box::new(dev));
                        }
                        Err(e) => error!("error adding mcp23017: {}", e),
                    };
                }
            }
        }
    }

    pub fn add_sensors_from_config(&mut self, sensor_config: HashMap<String, config::Sensor>) {
        let sensors = HashMap::new();

        for (name, config) in sensor_config.iter() {
            match config.device {
                config::SensorType::BMP085 { address, mode } => {
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

                    match bmp085::Device::new(address, self.i2c.clone(), trans_mode) {
                        Ok(dev) => self.add_device(name.to_string(), Box::new(dev)),
                        Err(e) => error!("error adding bmp085: {}", e),
                    };
                }
                config::SensorType::MCP9808 { address } => {
                    info!(
                        "Adding MCP9808 sensor named '{}' at i2c address {}",
                        name, address
                    );
                    match mcp9808::Device::new(address, self.i2c.clone()) {
                        Ok(dev) => self.add_device(name.to_string(), Box::new(dev)),
                        Err(e) => error!("error adding mcp9808: {}", e),
                    };
                }
            }
        }
    }
}

pub fn new() -> State {
    let dt = DateTime::local();
    let i2c = bus::start();

    State {
        i2c,
        dt,
        devices: HashMap::new(),
    }
}
