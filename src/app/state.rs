extern crate chrono;

use crate::app::{db, device, input, output, AppID};
use crate::config;
use crate::config::types::BoolExpr;
use crate::error::{Error, Result};
use crate::rpi;
use crate::rpi::device::Device;
use chrono::prelude::*;
use db::models;
use std::collections::HashMap;
use tracing::{error, info, instrument};

use super::dimensioned::Dimensioned;

/// Keep current app state in memory, together with device state
pub struct State {
    dt: DateTime<Local>,
    db: db::Db,
    devices: HashMap<AppID, rpi::device::Device>,

    /// Cached output automation compilations with a flag for mark/sweep
    output_automation_cache: HashMap<String, (bool, BoolExpr)>,

    i2c: rpi::RpiApi,
    here: (f64, f64),
}

// Internal State machine for the application. this is core logic.
impl State {
    pub fn lat(&self) -> f64 {
        self.here.0
    }

    pub fn long(&self) -> f64 {
        self.here.1
    }

    pub fn devices(&self) -> Result<Vec<device::Device>> {
        let models = self.db.devices()?;
        Ok(models
            .iter()
            .map(|d| device::Device {
                db_device: d.clone(),
            })
            .collect())
    }

    pub fn device_inputs(&self, device_id: &AppID) -> Result<Vec<input::Input>> {
        let models = self.db.inputs_for_device(device_id)?;
        Ok(models
            .iter()
            .map(|d| input::Input { db: d.clone() })
            .collect())
    }

    pub fn inputs(&self) -> Result<Vec<input::Input>> {
        let models = self.db.inputs()?;
        Ok(models
            .iter()
            .map(|d| input::Input { db: d.clone() })
            .collect())
    }

    pub fn device_outputs(&self, device_id: &AppID) -> Result<Vec<output::Output>> {
        let models = self.db.outputs_for_device(device_id)?;
        Ok(models
            .iter()
            .map(|d| output::Output { data: d.clone() })
            .collect())
    }

    pub fn outputs(&self) -> Result<Vec<output::Output>> {
        let models = self.db.outputs()?;
        Ok(models
            .iter()
            .map(|d| output::Output { data: d.clone() })
            .collect())
    }

    pub fn device(&self, name: &AppID) -> Result<device::Device> {
        let device = self.db.device(name)?;
        Ok(device::Device {
            db_device: device.clone(),
        })
    }

    pub fn device_slots(&self, name: &AppID) -> Result<Vec<device::Slot>> {
        let mdev = self
            .devices
            .get(name)
            .ok_or(Error::NonExistant("can't find device".to_string()))?;
        Ok(mdev.slots())
    }

    pub async fn add_device(
        &mut self,
        model: crate::app::device::Type,
        name: String,
        description: String,
        disabled: Option<bool>,
    ) -> Result<AppID> {
        let new_device = models::NewDevice::new(model, name, description, disabled);
        let db_device = self.db.add_device(&new_device)?;
        let model = serde_json::from_str(db_device.model.as_str())?;
        let id = db_device.name;
        let mut device = Device::new(model, self.i2c.clone());
        device.reset().await?;
        info!("Adding device id: {}", id);
        self.devices.insert(id.clone(), device);
        Ok(id.clone())
    }

    pub async fn add_input(&mut self, config: &models::NewInput) -> Result<AppID> {
        let mdev = self.devices.get_mut(&config.device_id);
        if let Some(_dev) = mdev {
            let db_input = self.db.add_input(config)?;
            Ok(db_input.name)
        } else {
            Err(Error::NonExistant(format!(
                "Could not add input to missing device {}",
                config.device_id
            )))
        }
    }

    pub async fn remove_input(&mut self, input_id: &AppID) -> Result<()> {
        self.db.remove_input(input_id)
    }

    pub async fn remove_output(&mut self, output_id: &AppID) -> Result<()> {
        self.db.remove_output(output_id)
    }

    pub async fn add_output(&mut self, config: &models::NewOutput) -> Result<AppID> {
        let mdev = self.devices.get_mut(&config.device_id);
        if let Some(_dev) = mdev {
            let db_output = self.db.add_output(config)?;
            Ok(db_output.name)
        } else {
            Err(Error::NonExistant(format!(
                "Could not add output to missing device {}",
                config.device_id
            )))
        }
    }

    pub async fn update_output(
        &mut self,
        output_id: AppID,
        fields: models::UpdateOutput,
    ) -> Result<AppID> {
        let _mout = self.db.update_output(&output_id, &fields)?;
        if let models::UpdateOutput {
            automation_script: Some(Some(q)),
            ..
        } = fields
        {
            self.output_automation_cache.remove(&q);
        }
        Ok(output_id)
    }

    pub async fn reset_device(&mut self, id: &AppID) -> Result<()> {
        let device = self.devices.get_mut(id).ok_or(Error::NonExistant(
            format!("reset_device: {}", id).to_string(),
        ))?;
        device.reset().await?;
        Ok(())
    }

    pub async fn remove_device(&mut self, name: &AppID) -> Result<()> {
        info!("Remove device: '{}'", name);
        self.db.remove_device(name)?;
        self.devices.remove(name);
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
     * read what is currently being outputed
     */
    pub async fn read_output_bool(&self, output_id: &AppID) -> Result<bool> {
        let output = self.db.output(output_id)?;

        if let Some(device) = self.devices.get(&output.device_id) {
            Ok(device.read_boolean(output.device_output_id).await?)
        } else {
            Err(Error::NonExistant("can't find device".to_string()))
        }
    }

    /**
     * read a named input
     */
    pub async fn read_input_bool(&self, input_id: &AppID) -> Result<bool> {
        let input = self.db.input(input_id)?;

        if let Some(device) = self.devices.get(&input.device_id) {
            Ok(device.read_boolean(input.device_input_id).await?)
        } else {
            Err(Error::NonExistant("can't find device".to_string()))
        }
    }

    /**
     * Write a particular value to an output
     */
    pub async fn write_output_bool(&mut self, output_id: &AppID, value: bool) -> Result<()> {
        let output = self.db.output(output_id)?;

        if let Some(device) = self.devices.get_mut(&output.device_id) {
            device
                .write_boolean(output.device_output_id, output.active_low ^ value)
                .await
        } else {
            Err(Error::NonExistant("can't find device".to_string()))
        }
    }

    /// recompile all automation scripts with a fresh cache
    #[instrument(skip(self))]
    pub async fn compile_automations(&mut self) -> Result<()> {
        self.output_automation_cache = HashMap::new();

        let outputs = self.db.outputs()?;
        for output in outputs {
            if let Some(str_expr) = &output.automation_script {
                match config::parse::bool_expr(str_expr) {
                    Ok(expr) => {
                        self.output_automation_cache
                            .insert(str_expr.clone(), (false, expr));
                    }
                    Err(e) => {
                        error!("error parsing: {}", e)
                    }
                }
            }
        }

        Ok(())
    }

    /// update automation script cache and emit automations
    #[instrument(skip(self))]
    pub async fn emit_automations(&mut self) -> Result<()> {
        // Clear mark on all entries
        for (mark, _) in self.output_automation_cache.values_mut() {
            *mark = false
        }

        let outputs = self.db.outputs()?;
        for output in outputs {
            if let Some(str_expr) = &output.automation_script {
                // get or update the cached boolean expression
                let expr: Option<BoolExpr> = if let Some((mark, expr)) =
                    self.output_automation_cache.get_mut(str_expr)
                {
                    *mark = true;
                    Some(expr.clone())
                } else {
                    match config::parse::bool_expr(str_expr) {
                        Ok(expr) => {
                            self.output_automation_cache
                                .insert(str_expr.clone(), (true, expr.clone()));
                            Some(expr)
                        }
                        Err(e) => {
                            error!("error parsing automation script: {}", e);
                            None
                        }
                    }
                };

                // evaluate the expression and write it to the right output
                if let Some(expr) = expr {
                    match config::boolean::evaluate(self, &expr).await {
                        Ok(result) => {
                            if let Err(e) = self.write_output_bool(&output.name, result).await {
                                error!("failed to write: {}", e);
                            }
                        }
                        Err(e) => error!("{:?} has an error: {}", expr, e),
                    }
                }
            }
        }

        let mut to_kill: Vec<String> = vec![];

        for (k, (mark, _)) in &self.output_automation_cache {
            if !mark {
                to_kill.push(k.clone());
            }
        }

        for k in to_kill {
            self.output_automation_cache.remove(&k);
        }

        Ok(())
    }

    pub async fn read_input_value(&self, input_id: &AppID) -> Result<Dimensioned> {
        let input = self.db.input(input_id)?;

        if let Some(device) = self.devices.get(&input.device_id) {
            Ok(device.read_sensor(input.device_input_id).await?)
        } else {
            Err(Error::NonExistant("can't find device".to_string()))
        }
    }
}

pub async fn new_state(bus: u8, here: (f64, f64), db: crate::app::db::Db) -> Result<State> {
    let dt = Local::now();
    let i2c = rpi::start(bus);

    let mut device_instances: HashMap<AppID, Device> = HashMap::new();

    let devices = db.devices()?;
    for db_device in &devices {
        let model = serde_json::from_str(&db_device.model)?;
        info!("Adding device {:?} named '{}'", model, db_device.name);
        let mut new_device = Device::new(model, i2c.clone());
        new_device.reset().await?;
        device_instances.insert(db_device.name.clone(), new_device);
    }

    let mut state = State {
        i2c,
        dt,
        db,
        output_automation_cache: HashMap::new(),
        devices: device_instances,
        here,
    };

    state.compile_automations().await?;

    Ok(state)
}
