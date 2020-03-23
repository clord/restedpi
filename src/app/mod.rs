extern crate chrono;

use crate::config;
use crate::config::value::Unit;
use crate::i2c::{bus, device::Device, error::Error, Result};
use crate::storage;
use crate::webapp::slugify::slugify;
use chrono::prelude::*;
use std::collections::HashMap;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    devices: HashMap<String, Device>,
    i2c: bus::I2cBus,
    storage: storage::Storage,
}

// unsafe impl Send for State {}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, config: &config::Device) -> Result<()> {
        let mut device = Device::new(config, self.i2c.clone());
        info!(
            "Adding device: '{}' at I2C address: {}",
            config.name, config.address
        );

        if cfg!(raspberry_pi) {
            // TODO: Real raspberry pi can reset, but this is for debugging
            device.reset()?;
        }

        let mut inc: usize = 0;
        while self.devices.contains_key(&slugify(&config.name, inc)) {
            inc += 1;
        }
        let sname = slugify(&config.name, inc);
        self.storage.set_device(&sname, config)?;
        self.devices.insert(sname, device);
        Ok(())
    }

    pub fn device(&self, name: &str) -> Result<&Device> {
        match self.devices.get(name) {
            Some(d) => Ok(d),
            None => Err(Error::NonExistant(name.to_string())),
        }
    }

    pub fn edit_device(&mut self, name: &str, config: &config::Device) -> Result<&Device> {
        match self.devices.get_mut(name) {
            Some(d) => {
                info!("Edit device: '{}'", name);
                d.set_config(config);
                if cfg!(raspberry_pi) {
                    d.reset()?;
                }
                self.storage.set_device(name, config)?;
                Ok(d)
            }
            None => Err(Error::NonExistant(name.to_string())),
        }
    }

    pub fn remove_device(&mut self, name: &str) {
        info!("Remove device: '{}'", name);
        self.storage.remove_device(name).unwrap();
        self.devices.remove(name);
    }

    pub fn devices(&mut self) -> &mut HashMap<String, Device> {
        &mut self.devices
    }

    pub fn reset(&mut self) -> Result<()> {
        self.devices.clear();
        for (sname, config) in self.storage.read_devices()? {
            let mut device = Device::new(&config, self.i2c.clone());
            info!(
                "Adding device: '{}' at I2C address: {}",
                config.name, config.address
            );
            if cfg!(raspberry_pi) {
                device.reset()?;
            }
            self.devices.insert(sname, device);
        }

        self.dt = Local::now();
        Ok(())
    }

    pub fn current_dt(&self) -> DateTime<Local> {
        self.dt
    }

    pub fn switch_count(&self, name: &str) -> Result<usize> {
        match self.devices.get(name) {
            Some(m) => Ok(m.switch_count()),
            None => Err(Error::NonExistant(name.to_string())),
        }
    }

    pub fn sensor_count(&self, name: &str) -> Result<usize> {
        match self.devices.get(name) {
            Some(m) => Ok(m.sensor_count()),
            None => Err(Error::NonExistant(name.to_string())),
        }
    }

    pub fn switch_set(&mut self, name: String, switch: usize, value: bool) -> Result<()> {
        match self.devices.get_mut(&name) {
            Some(m) => {
                m.write_switch(switch, value)?;
                Ok(())
            }
            None => Err(Error::NonExistant(name)),
        }
    }

    pub fn switch_toggle(&mut self, name: String, switch: usize) -> Result<()> {
        match self.devices.get_mut(&name) {
            Some(m) => {
                let value = m.read_sensor(switch)?;
                match value {
                    (cur_value, Unit::Boolean) => {
                        self.switch_set(name, switch, if cur_value > 0f64 { false } else { true })
                    }
                    _ => Err(Error::UnitError(Unit::Boolean)),
                }
            }
            None => Err(Error::NonExistant(name)),
        }
    }

    pub fn read_sensor(&mut self, name: String, sensor: usize) -> Result<(f64, Unit)> {
        match self.devices.get_mut(&name) {
            Some(m) => m.read_sensor(sensor),
            None => Err(Error::NonExistant(name)),
        }
    }
}

pub fn new(path: &std::path::Path) -> Result<State> {
    let dt = Local::now();
    let i2c = bus::start();

    info!("using database at {}", path.to_string_lossy());
    let storage = storage::open(path)?;

    Ok(State {
        i2c,
        dt,
        storage,
        devices: HashMap::new(),
    })
}
