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
    pub fn add_device(&mut self, config: &config::Device) {
        let device = Device::new(config, self.i2c.clone());
        info!(
            "Adding device: '{}' at I2C address: {}",
            config.name, config.address
        );
        let mut inc: usize = 0;
        while self.devices.contains_key(&slugify(&config.name, inc)) {
            inc += 1;
        }
        self.devices.insert(slugify(&config.name, inc), device);
    }

    pub fn remove_device(&mut self, name: &str) {
        info!("Remove device: '{}'", name);
        self.devices.remove(name);
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
