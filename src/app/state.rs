extern crate chrono;

use crate::config;
use crate::config::boolean::evaluate;
use crate::config::value::Unit;
use crate::i2c::{bus, device::Device, error::Error, Result};
use crate::storage;
use chrono::prelude::*;
use std::collections::HashMap;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    devices: HashMap<String, Device>,
    inputs: HashMap<String, config::Input>,
    outputs: HashMap<String, config::Output>,
    i2c: bus::I2cBus,
    storage: storage::Storage,
    bool_variables: HashMap<String, bool>,
}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, id: &str, config: &config::Device) -> Result<()> {
        let mut device = Device::new(config, self.i2c.clone());
        info!("Adding or replacing device with id: {}", id);

        if cfg!(raspberry_pi) {
            device.reset()?;
        }

        self.storage.set_device(&id, config)?;
        self.devices.insert(id.to_string(), device);

        Ok(())
    }

    pub fn device_config(
        &self,
        name: &str,
    ) -> Result<(
        config::Device,
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        match self.devices.get(name) {
            Some(d) => {
                let inputs = self.inputs_using_device(name);
                let outputs = self.outputs_using_device(name);
                Ok((d.config.clone(), inputs, outputs))
            }
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

    pub fn inputs_using_device(&self, id: &str) -> HashMap<String, config::Input> {
        let mut affected = HashMap::new();
        for (iid, input) in &self.inputs {
            match input {
                config::Input::FloatWithUnitFromDevice { device_id, .. } => {
                    if device_id == id {
                        affected.insert(iid.clone(), input.clone());
                    }
                }
                config::Input::BoolFromDevice { device_id, .. } => {
                    if device_id == id {
                        affected.insert(iid.clone(), input.clone());
                    }
                }
                // TODO: BoolExpr could refer to an input of this device!
                _ => (),
            }
        }
        affected
    }

    pub fn outputs_using_device(&self, id: &str) -> HashMap<String, config::Output> {
        let mut affected = HashMap::new();
        for (oid, output) in &self.outputs {
            match output {
                config::Output::BoolToDevice { device_id, .. } => {
                    if device_id == id {
                        affected.insert(oid.clone(), output.clone());
                    }
                }
                _ => (),
            }
        }
        affected
    }

    pub fn remove_device(
        &mut self,
        name: &str,
    ) -> Result<(
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        info!("Remove device: '{}'", name);

        // compute the list of inputs and outputs that need to be removed, and do that too.
        let afflicted_inputs = self.inputs_using_device(name);
        let afflicted_outputs = self.outputs_using_device(name);

        self.storage.remove_device(name)?;
        self.devices.remove(name);

        for o in &afflicted_outputs {
            self.remove_output(&o.0)?;
        }

        for i in &afflicted_inputs {
            self.remove_input(&i.0)?;
        }

        Ok((afflicted_inputs, afflicted_outputs))
    }

    pub fn remove_input(&mut self, id: &str) -> Result<()> {
        self.storage.remove_input(id)?;
        self.inputs.remove(id);
        Ok(())
    }
    pub fn remove_output(&mut self, id: &str) -> Result<()> {
        self.storage.remove_output(id)?;
        self.outputs.remove(id);
        Ok(())
    }

    pub fn devices(
        &mut self,
    ) -> HashMap<
        String,
        (
            config::Device,
            HashMap<String, config::Input>,
            HashMap<String, config::Output>,
        ),
    > {
        let mut result = HashMap::new();
        for (k, v) in &self.devices {
            let inputs = self.inputs_using_device(k);
            let outputs = self.outputs_using_device(k);
            result.insert(k.clone(), (v.config.clone(), inputs, outputs));
        }
        return result;
    }

    /**
     * Reset the whole app, setting up stuff from storage in addition to config
     */
    pub fn reset(&mut self) -> Result<()> {
        self.devices.clear();
        for (sname, config) in self.storage.all_devices()? {
            let mut device = Device::new(&config, self.i2c.clone());
            if cfg!(raspberry_pi) {
                device.reset()?;
            }
            self.devices.insert(sname, device);
        }

        self.inputs = self.storage.all_inputs()?;
        self.outputs = self.storage.all_outputs()?;

        self.dt = Local::now();
        Ok(())
    }

    /**
     * retrieve what the system thinks the current time and date is
     */
    pub fn current_dt(&self) -> DateTime<Local> {
        self.dt
    }

    /**
     * read a named input
     */
    pub fn read_input_bool(&self, input_id: &str) -> Result<bool> {
        let m_input = self.storage.get_input(input_id)?;
        match m_input.ok_or(Error::InputNotFound(input_id.to_owned()))? {
            config::Input::ExpressionResult(expr) => Ok(evaluate(self, &expr)),
            config::Input::BoolFromDevice {
                name: _,
                device_id,
                device_input_id,
                active_low: _,
            } => {
                let device_handle = self.devices.get(&device_id);
                let device = device_handle.ok_or(Error::NonExistant(device_id))?;
                let value = device.read_boolean(device_input_id)?;
                Ok(value)
            }
            config::Input::BoolFromVariable => self
                .bool_variables
                .get(input_id)
                .cloned()
                .ok_or(Error::NonExistant(input_id.to_string())),
            config::Input::FloatWithUnitFromDevice { .. } => {
                // TODO: Could read the float and convert to boolean using thresholds in config...
                Err(Error::UnitError(Unit::Boolean))
            }
        }
    }

    /**
     * Write a particular value to an output
     */
    pub fn write_output_bool(&mut self, output_id: &str, value: bool) -> Result<()> {
        let m_output = self.storage.get_output(output_id)?;
        let input = m_output.ok_or(Error::OutputNotFound(output_id.to_owned()))?;
        match input {
            config::Output::BoolToDevice {
                device_id,
                device_output_id,
                ..
            } => {
                let device_handle = self.devices.get_mut(&device_id);
                let device = device_handle.ok_or(Error::NonExistant(device_id))?;
                device.write_boolean(device_output_id, value)?;
                Ok(())
            }
            config::Output::BoolToVariable => match self.bool_variables.get_mut(output_id) {
                Some(var) => {
                    *var = value;
                    Ok(())
                }
                None => Err(Error::NonExistant(output_id.to_string())),
            },
        }
    }

    pub fn read_sensor(&self, device_id: &str, sensor_id: usize) -> Result<(f64, Unit)> {
        match self.devices.get(device_id) {
            Some(m) => m.read_sensor(sensor_id),
            None => Err(Error::NonExistant(device_id.to_string())),
        }
    }
}

pub fn new(config: config::Config) -> Result<State> {
    let dt = Local::now();
    let i2c = bus::start();

    let path = config
        .database
        .unwrap_or(std::path::PathBuf::from("rested-pi.db"));
    info!("using database at {}", path.to_string_lossy());
    let storage = storage::open(&path)?;

    let mut state = State {
        i2c,
        dt,
        storage,
        devices: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        bool_variables: HashMap::new(),
    };
    if let Some(device_config) = config.devices {
        for (name, device_config) in device_config.iter() {
            state
                .add_device(name, device_config)
                .expect("pre-configured device to not fail to reset");
        }
    }

    state.reset()?;

    Ok(state)
}
