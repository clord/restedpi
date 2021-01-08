extern crate chrono;

use crate::config;
use crate::config::Unit;
use crate::error::{Error, Result};
use crate::rpi;
use crate::rpi::device::Device;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,

    devices: HashMap<String, Device>,
    device_configs: HashMap<String, config::Device>,
    devices_change: Sender<HashMap<String, config::Device>>,

    inputs: HashMap<String, config::Input>,
    inputs_change: Sender<HashMap<String, config::Input>>,

    outputs: HashMap<String, config::Output>,
    outputs_change: Sender<HashMap<String, config::Output>>,

    i2c: rpi::RpiApi,
    here: (f64, f64),
}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn add_device(&mut self, id: &str, config: config::Device) -> Result<()> {
        let mut device = Device::new(config.clone(), self.i2c.clone());
        info!("Adding or replacing device with id: {}", id);
        device.reset()?;
        self.devices.insert(id.to_string(), device);
        self.devices_change
            .send(self.device_configs.clone())
            .unwrap_or(());
        Ok(())
    }

    pub fn lat(&self) -> f64 {
        self.here.0
    }
    pub fn long(&self) -> f64 {
        self.here.1
    }

    pub fn add_input(&mut self, id: &str, config: &config::Input) -> Result<()> {
        self.inputs.insert(id.to_string(), config.clone());
        self.inputs_change.send(self.inputs.clone()).unwrap_or(());
        Ok(())
    }

    pub fn add_output(&mut self, id: &str, config: &config::Output) -> Result<()> {
        self.outputs.insert(id.to_string(), config.clone());
        self.outputs_change.send(self.outputs.clone()).unwrap_or(());
        Ok(())
    }

    pub fn reset_device(&mut self, id: &str) -> Result<()> {
        let device = self.devices.get_mut(id).ok_or(Error::NonExistant(
            format!("reset_device: {}", id).to_string(),
        ))?;
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
        match self.device_configs.get(name) {
            Some(config) => {
                let inputs = self.inputs_using_device(name);
                let outputs = self.outputs_using_device(name);
                Ok((config.clone(), inputs, outputs))
            }
            None => Err(Error::NonExistant(
                format!("device config for {}", name).to_string(),
            )),
        }
    }

    pub fn inputs_using_device(&self, id: &str) -> HashMap<String, config::Input> {
        let mut affected = HashMap::new();
        for (iid, input) in &self.inputs {
            if input.device_id == id {
                affected.insert(iid.clone(), input.clone());
            }
        }
        affected
    }

    pub fn outputs_using_device(&self, id: &str) -> HashMap<String, config::Output> {
        let mut affected = HashMap::new();
        for (oid, output) in &self.outputs {
            if output.device_id == id {
                affected.insert(oid.clone(), output.clone());
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

        self.devices.remove(name);
        self.devices_change
            .send(self.device_configs.clone())
            .unwrap_or(());

        for o in &afflicted_outputs {
            self.remove_output(&o.0)?;
        }

        for i in &afflicted_inputs {
            self.remove_input(&i.0)?;
        }

        Ok((afflicted_inputs, afflicted_outputs))
    }

    pub fn remove_input(&mut self, id: &str) -> Result<()> {
        self.inputs.remove(id);
        self.inputs_change.send(self.inputs.clone()).unwrap_or(());
        Ok(())
    }

    pub fn remove_output(&mut self, id: &str) -> Result<()> {
        self.outputs.remove(id);
        self.outputs_change.send(self.outputs.clone()).unwrap_or(());
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
        for (k, config) in &self.device_configs {
            let inputs = self.inputs_using_device(k);
            let outputs = self.outputs_using_device(k);
            result.insert(k.clone(), (config.clone(), inputs, outputs));
        }
        return result;
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
        let m_input = self.inputs.get(input_id);

        let config::Input {
            device_id,
            device_input_id,
            unit,
            ..
        } = m_input.ok_or(Error::InputNotFound(input_id.to_owned()))?;
        let device = self
            .devices
            .get(device_id)
            .ok_or(Error::NonExistant(format!(
                "read_input_bool: {}",
                device_id
            )))?;
        if *unit != Unit::Boolean {
            warn!("Can't read {:?}  from input {}", unit, input_id);
            return Err(Error::UnitError("can't read".to_string()));
        }
        let value = device.read_boolean(*device_input_id)?;
        Ok(value)
    }

    /**
     * Write a particular value to an output
     */
    pub fn write_output_bool(&mut self, output_id: &str, value: bool) -> Result<()> {
        let m_output = self.outputs.get(output_id);
        let output = m_output.ok_or(Error::OutputNotFound(output_id.to_owned()))?;
        let config::Output {
            device_id,
            active_low,
            unit,
            device_output_id,
            ..
        } = output;

        let device = self
            .devices
            .get_mut(device_id)
            .ok_or(Error::NonExistant(format!(
                "write_output_bool: {}",
                device_id
            )))?;

        if *unit != Unit::Boolean {
            warn!("Can't write {:?} to output {}", unit, output_id);
            return Err(Error::UnitError("can't write".to_string()));
        }

        device.write_boolean(*device_output_id, active_low.unwrap_or(false) ^ value)
    }

    pub fn emit_automations(&mut self) {
        let keys: Vec<String> = { self.outputs.keys().cloned().collect() };
        for output_id in keys {
            debug!("automation for {}", output_id);
            let output = { self.outputs.get(&output_id).cloned() };
            if let Some(config::Output { on_when, .. }) = output {
                if let Some(expr) = on_when {
                    // TODO: Parse expr from string!
                    match config::parse::bool_expr(&expr) {
                        Ok(parsed) => match config::boolean::evaluate(self, &parsed) {
                            Ok(result) => {
                                if let Err(e) = self.write_output_bool(&output_id, result) {
                                    error!("failed to write: {}", e);
                                }
                            }
                            Err(e) => error!("{:?} has an error: {}", expr, e),
                        },
                        Err(_) => error!("error parsing"),
                    }
                }
            }
        }
    }

    pub fn read_input_value(&self, input_id: &str) -> Result<(f64, Unit)> {
        let m_input = self.inputs.get(input_id);
        let config::Input {
            device_id,
            device_input_id,
            unit,
            ..
        } = m_input.ok_or(Error::InputNotFound(input_id.to_owned()))?;
        let device_handle = self.devices.get(device_id);
        let device = device_handle.ok_or(Error::NonExistant(
            format!("read_input_value: {}", device_id).to_string(),
        ))?;
        match unit {
            Unit::Boolean => {
                let value = device.read_boolean(*device_input_id)?;
                Ok(if value {
                    (1.0, config::Unit::Boolean)
                } else {
                    (0.0, config::Unit::Boolean)
                })
            }
            _ => device.read_sensor(*device_input_id),
        }
    }

    pub fn read_sensor(&self, device_id: &str, sensor_id: u32) -> Result<(f64, Unit)> {
        match self.devices.get(device_id) {
            Some(m) => m.read_sensor(sensor_id),
            None => Err(Error::NonExistant(
                format!("read_sensor: {}", device_id).to_string(),
            )),
        }
    }
}

pub fn new_state(
    here: (f64, f64),
    devices: HashMap<String, config::Device>,
    devices_change: Sender<HashMap<String, config::Device>>,

    inputs: HashMap<String, config::Input>,
    inputs_change: Sender<HashMap<String, config::Input>>,

    outputs: HashMap<String, config::Output>,
    outputs_change: Sender<HashMap<String, config::Output>>,
) -> Result<State> {
    let dt = Local::now();
    let i2c = rpi::start();
    let mut device_instances: HashMap<String, Device> = HashMap::new();

    for (k, cfg) in &devices {
        info!("adding device: {}", cfg.name);
        device_instances.insert(k.clone(), Device::new(cfg.clone(), i2c.clone()));
    }

    let state = State {
        i2c,
        dt,

        device_configs: devices,
        devices: device_instances,
        devices_change,

        here,

        inputs,
        inputs_change,

        outputs,
        outputs_change,

    };

    Ok(state)
}
