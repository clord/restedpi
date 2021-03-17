extern crate chrono;

use crate::config;
use crate::app::AppID;
use crate::app::device;
use crate::app::input::Input;
use crate::app::output::Output;
use crate::error::{Error, Result};
use crate::rpi;
use crate::rpi::device::Device;
use crate::app::db;
use chrono::prelude::*;

use std::collections::HashMap;
use tracing::{ instrument, error, info, warn};


// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    db: db::Db,
    devices: HashMap<AppID, device::Device>,
    i2c: rpi::RpiApi,
    here: (f64, f64),
}

// Internal State machine for the application. this is core logic.
impl State {
    pub async fn add_device(&mut self, id: AppID, config: db::Device) -> Result<()> {
        let mut device = Device::new(config.clone(), self.i2c.clone());
        info!("Adding or replacing device with id: {}", id);
        device.reset().await?;
        self.devices.insert(id, device);
        // TODO: add-or-replace device in database
        Ok(())
    }

    pub fn lat(&self) -> f64 {
        self.here.0
    }
    pub fn long(&self) -> f64 {
        self.here.1
    }

    pub async fn add_input(&mut self, id: AppID, config: &db::Input) -> Result<()> {
        self.inputs.insert(id, config.clone());
        // TODO: add-or-replace in database
        Ok(())
    }

    pub async fn add_output(&mut self, id: AppID, config: &db::Output) -> Result<()> {
        self.outputs.insert(id, config.clone());
        // TODO: add-or-replace in database
        Ok(())
    }

    pub async fn reset_device(&mut self, id: AppID) -> Result<()> {
        let device = self.devices.get_mut(id).ok_or(Error::NonExistant(
            format!("reset_device: {}", id).to_string(),
        ))?;
        device.reset().await?;
        Ok(())
    }

    pub fn device_config(
        &self,
        name: AppID,
    ) -> Result<(
        Device,
        HashMap<AppID, Input>,
        HashMap<AppID, Output>,
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

    pub fn inputs_using_device(&self, id: AppID) -> HashMap<AppID, Input> {
        let mut affected = HashMap::new();
        for (iid, input) in &self.inputs {
            if input.device_id == id {
                affected.insert(iid.clone(), input.clone());
            }
        }
        affected
    }

    pub fn outputs_using_device(&self, id: AppID) -> HashMap<AppID, Output> {
        let mut affected = HashMap::new();
        for (oid, output) in &self.outputs {
            if output.device_id == id {
                affected.insert(oid.clone(), output.clone());
            }
        }
        affected
    }

    pub async fn remove_device(
        &mut self,
        name: AppID,
    ) -> Result<(
        HashMap<AppID, Input>,
        HashMap<AppID, Output>,
    )> {
        info!("Remove device: '{}'", name);

        // compute the list of inputs and outputs that need to be removed, and do that too.
        let afflicted_inputs = self.inputs_using_device(name);
        let afflicted_outputs = self.outputs_using_device(name);

        self.devices.remove(name);

        for o in afflicted_outputs {
            // TODO: should start them all, tben await them all
            self.remove_output(o.0).await?;
        }

        for i in afflicted_inputs {
            self.remove_input(i.0).await?;
        }

        Ok((afflicted_inputs, afflicted_outputs))
    }

    pub async fn remove_input(&mut self, id: AppID) -> Result<()> {
        self.inputs().remove(id);
        Ok(())
    }

    pub async fn remove_output(&mut self, id: AppID) -> Result<()> {
        self.outputs().remove(id);
        Ok(())
    }

    pub fn outputs(&self) -> &HashMap<AppID, Output> {
        &self.outputs
    }

    pub fn inputs(&self) -> &HashMap<AppID, Input> {
        &self.inputs
    }

    pub fn devices(
        &self,
    ) -> HashMap<
        AppID,
        (
            Device,
            HashMap<AppID, Input>,
            HashMap<AppID, Output>,
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
     * read what is currently being outputed
     */
    pub async fn read_output_bool(&self, output_id: &str) -> Result<bool> {
        let m_output = self.outputs.get(output_id);

        let Output {
            device_id,
            unit,
            device_output_id,
            ..
        } = m_output.ok_or(Error::OutputNotFound(output_id.to_owned()))?;

        let device = self
            .devices
            .get(device_id)
            .ok_or(Error::NonExistant(format!(
                "read_output_bool: {}",
                device_id
            )))?;

        if *unit != device::Unit::Boolean {
            warn!("Can't read {:?}  from output {}", unit, output_id);
            return Err(Error::UnitError("can't read".to_string()));
        }
        let value = device.read_boolean(*device_output_id).await?;
        Ok(value)
    }

    /**
     * read a named input
     */
    pub async fn read_input_bool(&self, input_id: AppID) -> Result<bool> {
        let m_input = self.inputs.get(input_id);

        let Input {
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
        if *unit != device::Unit::Boolean {
            warn!("Can't read {:?}  from input {}", unit, input_id);
            return Err(Error::UnitError("can't read".to_string()));
        }
        let value = device.read_boolean(*device_input_id).await?;
        Ok(value)
    }

    /**
     * Write a particular value to an output
     */
    pub async fn write_output_bool(&mut self, output_id: AppID, value: bool) -> Result<()> {
        let m_output = self.outputs.get(output_id);
        let output = m_output.ok_or(Error::OutputNotFound(output_id.to_owned()))?;
        let Output {
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

        if *unit != device::Unit::Boolean {
            warn!("Can't write {:?} to output {}", unit, output_id);
            return Err(Error::UnitError("can't write".to_string()));
        }

        device
            .write_boolean(*device_output_id, active_low.unwrap_or(false) ^ value)
            .await
    }

    #[instrument(skip(self))]
    pub async fn emit_automation(&mut self, output_id: &str) {
        let output = { self.outputs.get(output_id).cloned() };
        if let Some(Output { on_when, .. }) = output {
            if let Some(expr) = on_when {
                match crate::config::parse::bool_expr(&expr) {
                    Ok(parsed) => match config::boolean::evaluate(self, &parsed).await {
                        Ok(result) => {
                            if let Err(e) = self.write_output_bool(&output_id, result).await {
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

    pub async fn emit_automations(&mut self) {
        let keys: Vec<String> = { self.outputs().keys().cloned().collect() };
        for output_id in keys {
            self.emit_automation(&output_id).await;
        }
    }

    pub async fn read_input_value(&self, input_id: &str) -> Result<(f64, device::Unit)> {
        let m_input = self.inputs().get(input_id);
        let Input {
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
            device::Unit::Boolean => {
                let value = device.read_boolean(*device_input_id).await?;
                Ok(if value {
                    (1.0, device::Unit::Boolean)
                } else {
                    (0.0, device::Unit::Boolean)
                })
            }
            _ => device.read_sensor(*device_input_id).await,
        }
    }

    pub async fn read_sensor(&self, device_id: AppID, sensor_id: u32) -> Result<(f64, device::Unit)> {
        match self.devices.get(&device_id) {
            Some(m) => m.read_sensor(sensor_id).await,
            None => Err(Error::NonExistant(
                format!("read_sensor: {}", device_id).to_string(),
            )),
        }
    }
}

pub async fn new_state(
    here: (f64, f64),
    db: crate::app::db::Db
) -> Result<State> {
    let dt = Local::now();
    let i2c = rpi::start();

    let mut device_instances: HashMap<AppID, Device> = HashMap::new();

    // TODO: Bring up our runtime environment, which bootstraps from db, then updates db as things
    // change. we only operate out of runtime environment normally, which has compiled expressions,
    // etc. runtime environment is also where we pull queries from.

    let devices = db.devices()?;
    for dbDevice in &devices {
        info!("adding device: {}", dbDevice.name);
        device_instances.insert(dbDevice.device_id, Device::new(dbDevice.clone(), i2c.clone()));
    }

    let state = State {
        i2c,
        dt,
        db,
        devices: device_instances,
        here,
    };

    Ok(state)
}
