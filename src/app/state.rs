extern crate chrono;

use crate::app::{AppID, db, device, input, output};
use crate::config;
use crate::config::types::BoolExpr;
use crate::error::{Error, Result};
use crate::rpi;
use crate::rpi::device::Device;
use chrono::prelude::*;
use db::models;
use std::collections::HashMap;
use tracing::{error, info, instrument, warn};

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
        // Verify the hardware actually works before persisting anything: a
        // device that fails reset must not be stored, or it would haunt every
        // subsequent boot and leave the DB and in-memory state out of sync.
        let mut device = Device::new(model, self.i2c.clone());
        device.reset().await?;
        let new_device = models::NewDevice::new(model, name, description, disabled);
        let db_device = self.db.add_device(&new_device)?;
        let id = db_device.name;
        info!("Adding device id: {}", id);
        self.devices.insert(id.clone(), device);
        Ok(id)
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
     * Verify a device exists and is not disabled; error otherwise
     */
    fn ensure_device_enabled(&self, device_id: &AppID) -> Result<()> {
        let device = self.db.device(device_id)?;
        if device.disabled {
            Err(Error::DeviceDisabled(device_id.clone()))
        } else {
            Ok(())
        }
    }

    /**
     * read what is currently being outputed
     *
     * Applies `active_low` to the raw device value so the logical value written
     * via `write_output_bool` round-trips.
     */
    pub async fn read_output_bool(&self, output_id: &AppID) -> Result<bool> {
        let output = self.db.output(output_id)?;
        self.ensure_device_enabled(&output.device_id)?;

        if let Some(device) = self.devices.get(&output.device_id) {
            let raw = device.read_boolean(output.device_output_id).await?;
            Ok(output.active_low ^ raw)
        } else {
            Err(Error::NonExistant("can't find device".to_string()))
        }
    }

    /**
     * read a named input
     */
    pub async fn read_input_bool(&self, input_id: &AppID) -> Result<bool> {
        let input = self.db.input(input_id)?;
        self.ensure_device_enabled(&input.device_id)?;

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
        self.ensure_device_enabled(&output.device_id)?;

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

        let disabled_devices: std::collections::HashSet<AppID> = self
            .db
            .devices()?
            .into_iter()
            .filter(|d| d.disabled)
            .map(|d| d.name)
            .collect();

        let outputs = self.db.outputs()?;
        for output in outputs {
            if disabled_devices.contains(&output.device_id) {
                continue;
            }
            if let Some(str_expr) = &output.automation_script {
                // get or update the cached boolean expression
                let expr: Option<BoolExpr> =
                    if let Some((mark, expr)) = self.output_automation_cache.get_mut(str_expr) {
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
                        Err(e) => {
                            warn!("Automation expression {:?} evaluation failed: {}", expr, e)
                        }
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
        self.ensure_device_enabled(&input.device_id)?;

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

    // A single broken device (corrupt model JSON, unplugged sensor, wrong I2C
    // address, ...) must not prevent the server from booting: log it, skip it,
    // and keep going. Reads of a skipped device return NonExistant errors.
    let devices = db.devices()?;
    for db_device in &devices {
        let model: device::Type = match serde_json::from_str(&db_device.model) {
            Ok(model) => model,
            Err(e) => {
                error!(
                    "Skipping device '{}': stored model is not valid JSON: {}",
                    db_device.name, e
                );
                continue;
            }
        };
        info!("Adding device {:?} named '{}'", model, db_device.name);
        let mut new_device = Device::new(model, i2c.clone());
        match new_device.reset().await {
            Ok(()) => {
                device_instances.insert(db_device.name.clone(), new_device);
            }
            Err(e) => {
                error!(
                    "Skipping device '{}': hardware reset failed: {}",
                    db_device.name, e
                );
            }
        }
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

#[cfg(all(test, feature = "mock-gpio"))]
mod mock_gpio_tests {
    use super::*;
    use crate::app::device::{Directions, MCP23017, Type};

    async fn fresh_state(tag: &str) -> State {
        let dir = std::env::temp_dir().join(format!(
            "restedpi-state-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let db = db::Db::start_db(&dir).expect("failed to create test database");
        new_state(1, (0.0, 0.0), db)
            .await
            .expect("failed to create state")
    }

    fn gpio_expander() -> Type {
        Type::MCP23017(MCP23017 {
            address: 0x20,
            bank_a: Directions::new(),
            bank_b: Directions::new(),
        })
    }

    #[tokio::test]
    async fn active_low_output_round_trips_logical_value() {
        let mut state = fresh_state("active-low").await;
        state
            .add_device(gpio_expander(), "dev".to_string(), String::new(), None)
            .await
            .expect("failed to add device");
        state
            .add_output(&models::NewOutput::new(
                "out".to_string(),
                "dev".to_string(),
                0,
                true, // active_low
                None,
            ))
            .await
            .expect("failed to add output");

        let out_id = "out".to_string();

        state.write_output_bool(&out_id, true).await.unwrap();
        assert!(
            state.read_output_bool(&out_id).await.unwrap(),
            "logical true should read back as true on an active-low output"
        );

        state.write_output_bool(&out_id, false).await.unwrap();
        assert!(
            !state.read_output_bool(&out_id).await.unwrap(),
            "logical false should read back as false on an active-low output"
        );
    }

    #[tokio::test]
    async fn active_high_output_round_trips_logical_value() {
        let mut state = fresh_state("active-high").await;
        state
            .add_device(gpio_expander(), "dev".to_string(), String::new(), None)
            .await
            .expect("failed to add device");
        state
            .add_output(&models::NewOutput::new(
                "out".to_string(),
                "dev".to_string(),
                0,
                false, // active high
                None,
            ))
            .await
            .expect("failed to add output");

        let out_id = "out".to_string();

        state.write_output_bool(&out_id, true).await.unwrap();
        assert!(state.read_output_bool(&out_id).await.unwrap());

        state.write_output_bool(&out_id, false).await.unwrap();
        assert!(!state.read_output_bool(&out_id).await.unwrap());
    }

    #[tokio::test]
    async fn disabled_device_rejects_reads_and_writes() {
        let mut state = fresh_state("disabled").await;
        state
            .add_device(
                gpio_expander(),
                "dev".to_string(),
                String::new(),
                Some(true),
            )
            .await
            .expect("failed to add device");
        state
            .add_output(&models::NewOutput::new(
                "out".to_string(),
                "dev".to_string(),
                0,
                false,
                None,
            ))
            .await
            .expect("failed to add output");
        state
            .add_input(&models::NewInput::new(
                "in".to_string(),
                "dev".to_string(),
                1,
            ))
            .await
            .expect("failed to add input");

        let out_id = "out".to_string();
        let in_id = "in".to_string();

        assert_eq!(
            state.write_output_bool(&out_id, true).await,
            Err(Error::DeviceDisabled("dev".to_string()))
        );
        assert_eq!(
            state.read_output_bool(&out_id).await,
            Err(Error::DeviceDisabled("dev".to_string()))
        );
        assert_eq!(
            state.read_input_bool(&in_id).await,
            Err(Error::DeviceDisabled("dev".to_string()))
        );
        assert!(matches!(
            state.read_input_value(&in_id).await,
            Err(Error::DeviceDisabled(_))
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::db::Db;
    use std::path::PathBuf;

    /// Unique temporary directory for an isolated test database.
    fn temp_db_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before the unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "restedpi-state-test-{}-{}-{}",
            tag,
            std::process::id(),
            nanos
        ))
    }

    fn mcp9808_model() -> device::Type {
        device::Type::MCP9808(device::MCP9808 { address: 0x18 })
    }

    /// A device row with invalid model JSON must not prevent boot: the device
    /// is skipped (reads return NonExistant) while valid devices still load.
    #[tokio::test]
    async fn new_state_skips_device_with_invalid_model_json() {
        let dir = temp_db_dir("invalid-model");
        let db = Db::start_db(&dir).expect("create test db");

        db.execute_sql_for_tests(
            "INSERT INTO devices (name, model, notes) VALUES ('broken', 'not valid json', '')",
        )
        .expect("insert corrupt device row");
        let good = models::NewDevice::new(
            mcp9808_model(),
            "good".to_string(),
            "a working sensor".to_string(),
            None,
        );
        db.add_device(&good).expect("insert valid device row");

        let state = new_state(1, (0.0, 0.0), db)
            .await
            .expect("boot must succeed despite the corrupt device row");

        // Both rows are still in the DB...
        let db_devices = state.devices().expect("list devices");
        assert_eq!(db_devices.len(), 2);

        // ...but only the valid device made it into memory.
        assert!(state.device_slots(&"good".to_string()).is_ok());
        match state.device_slots(&"broken".to_string()) {
            Err(Error::NonExistant(_)) => (),
            other => panic!("expected NonExistant for skipped device, got {:?}", other),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A device whose hardware reset fails must be skipped at boot, not abort
    /// startup. Only meaningful in the stub build, where I2C access errors.
    #[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
    #[tokio::test]
    async fn new_state_skips_device_whose_reset_fails() {
        let dir = temp_db_dir("reset-fails");
        let db = Db::start_db(&dir).expect("create test db");

        // BMP085 reset reads calibration data over I2C, which fails in the
        // stub build; MCP9808 reset is a no-op and succeeds.
        let bmp = models::NewDevice::new(
            device::Type::BMP085(device::BMP085 {
                address: 0x77,
                mode: device::SamplingMode::Standard,
            }),
            "unreachable".to_string(),
            "sensor that fails reset".to_string(),
            None,
        );
        db.add_device(&bmp).expect("insert bmp085 row");
        let good = models::NewDevice::new(
            mcp9808_model(),
            "good".to_string(),
            "a working sensor".to_string(),
            None,
        );
        db.add_device(&good).expect("insert valid device row");

        let state = new_state(1, (0.0, 0.0), db)
            .await
            .expect("boot must succeed despite the failing device");

        assert!(state.device_slots(&"good".to_string()).is_ok());
        match state.device_slots(&"unreachable".to_string()) {
            Err(Error::NonExistant(_)) => (),
            other => panic!("expected NonExistant for skipped device, got {:?}", other),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// add_device must not persist a device whose hardware reset fails.
    /// Only meaningful in the stub build, where I2C access errors.
    #[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
    #[tokio::test]
    async fn add_device_does_not_persist_when_reset_fails() {
        let dir = temp_db_dir("add-reset-fails");
        let db = Db::start_db(&dir).expect("create test db");
        let mut state = new_state(1, (0.0, 0.0), db).await.expect("boot empty db");

        let result = state
            .add_device(
                device::Type::BMP085(device::BMP085 {
                    address: 0x77,
                    mode: device::SamplingMode::Standard,
                }),
                "unreachable".to_string(),
                "sensor that fails reset".to_string(),
                None,
            )
            .await;
        assert!(result.is_err(), "reset failure must propagate to caller");

        // Nothing was persisted, so the next boot is unaffected.
        let db_devices = state.devices().expect("list devices");
        assert!(db_devices.is_empty(), "failed device must not be stored");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// add_device persists and registers the device once reset succeeds.
    #[tokio::test]
    async fn add_device_persists_after_successful_reset() {
        let dir = temp_db_dir("add-ok");
        let db = Db::start_db(&dir).expect("create test db");
        let mut state = new_state(1, (0.0, 0.0), db).await.expect("boot empty db");

        let id = state
            .add_device(
                mcp9808_model(),
                "temp1".to_string(),
                "a temperature sensor".to_string(),
                None,
            )
            .await
            .expect("adding a healthy device succeeds");

        assert_eq!(id, "temp1");
        assert!(state.device_slots(&id).is_ok());
        let db_devices = state.devices().expect("list devices");
        assert_eq!(db_devices.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
