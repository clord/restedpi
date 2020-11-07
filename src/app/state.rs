extern crate chrono;

use crate::config;
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

    pub fn add_input(&mut self, id: &str, config: &config::Input) -> Result<()> {
        self.storage.set_input(&id, &config)?;
        self.inputs.insert(id.to_string(), config.clone());
        Ok(())
    }

    pub fn add_output(&mut self, id: &str, config: &config::Output) -> Result<()> {
        self.storage.set_output(&id, &config)?;
        self.outputs.insert(id.to_string(), config.clone());
        Ok(())
    }

    pub fn reset_device(&mut self, id: &str) -> Result<()> {
        let device = self
            .devices
            .get_mut(id)
            .ok_or(Error::NonExistant(id.to_string()))?;
        device.reset()?;
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
     * set current dt to a partcular time
     */
    pub fn set_current_dt(&mut self, new_dt: DateTime<Local>) {
        self.dt = new_dt;
    }

    /**
     * read a named input
     */
    pub fn read_input_bool(&self, input_id: &str) -> Result<bool> {
        let m_input = self.storage.get_input(input_id)?;
        match m_input.ok_or(Error::InputNotFound(input_id.to_owned()))? {
            config::Input::BoolFromDevice {
                name: _,
                device_id,
                device_input_id,
                active_low: _,
            } => {
                debug!("will read!");
                let device_handle = self.devices.get(&device_id);
                let device = device_handle.ok_or(Error::NonExistant(device_id))?;
                let value = device.read_boolean(device_input_id)?;
                debug!("did read!");
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
        let output = m_output.ok_or(Error::OutputNotFound(output_id.to_owned()))?;
        match output {
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

    pub fn emit_automations(&mut self, app_channel: &crate::app::channel::AppChannel) {
        let keys: Vec<String> = { self.outputs.keys().cloned().collect() };
        for output_id in keys {
            debug!("automation for {}", output_id);
            let output = { self.outputs.get(&output_id).cloned() };
            if let Some(config::Output::BoolToDevice { automation, .. }) = output {
                if let Some(expr) = automation {
                    match config::boolean::evaluate(app_channel, &expr) {
                        Ok(result) => {
                            if let Err(e) = self.write_output_bool(&output_id, result) {
                                error!("failed to write: {}", e);
                            }
                        }
                        Err(e) => error!("{:?} has an error: {}", expr, e),
                    }
                }
            }
        }
    }

    pub fn read_input_value(&self, input_id: &str) -> Result<(f64, Unit)> {
        let m_input = self.storage.get_input(input_id)?;
        match m_input.ok_or(Error::InputNotFound(input_id.to_owned()))? {
            config::Input::BoolFromDevice {
                name: _,
                device_id,
                device_input_id,
                active_low: _,
            } => {
                let device_handle = self.devices.get(&device_id);
                let device = device_handle.ok_or(Error::NonExistant(device_id))?;
                let value = device.read_boolean(device_input_id)?;
                Ok(if value {
                    (1.0, config::Unit::Boolean)
                } else {
                    (0.0, config::Unit::Boolean)
                })
            }

            config::Input::BoolFromVariable => {
                let value = self
                    .bool_variables
                    .get(input_id)
                    .cloned()
                    .ok_or(Error::NonExistant(input_id.to_string()))?;

                Ok(if value {
                    (1.0, config::Unit::Boolean)
                } else {
                    (0.0, config::Unit::Boolean)
                })
            }

            config::Input::FloatWithUnitFromDevice {
                name: _,
                device_id,
                device_input_id,
            } => self.read_sensor(&device_id, device_input_id),
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

    if let Some(input_config) = config.inputs {
        for (name, input_config) in input_config.iter() {
            state
                .add_input(name, input_config)
                .expect("pre-configured device to not fail to reset");
        }
    }

    if let Some(output_config) = config.outputs {
        for (name, output_config) in output_config.iter() {
            state
                .add_output(name, output_config)
                .expect("pre-configured device to not fail to reset");
        }
    }

    state.reset()?;

    Ok(state)
}
