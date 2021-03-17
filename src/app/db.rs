use chrono::prelude::*;
use crate::error::{Result, Error};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

fn get_pool(db_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);

    Pool::new(manager).expect("Failed to create DB Pool")
}

pub struct Db {
    uri: String,
    db: DbPool,
}

/**
 * A device has some inputs and outputs, and connects to physical interfaces like gpio.
 */
#[derive(Queryable, Clone, Debug)]
pub struct Device {
    pub device_id: i32,

    /// What model of device is this? must be a supported type.
    pub model_type: String,

    /// configuraiton as json kept in the database. model-specific structure.
    pub model_config: String,

    /// What do we name this particular device?
    pub name: String,

    /// information about the device that we might need
    pub notes: String,

    /// If disabled, device will not be considered for certain operations
    pub disabled: bool,

    /// When was this created
    pub created_at: NaiveDateTime
}


/// Represent a particular input, meaning a source of information from a device.
#[derive(Queryable, Clone, Debug)]
pub struct Input {
    pub input_id: i32,

    /// What do we want to call this input
    pub name: String,

    /// The device this input is associated with
    pub device_id: i32,

    /// Each device can have multiple inputs and outputs, this is a device-specific index. (pin
    /// number, channel, etc)
    pub device_input_id: i32,

    /// what is the type of data that this input will produce ("Boolean", "DegC", etc)
    pub unit: String,

    /// When was this created
    pub created_at: NaiveDateTime
}

/// Represent a particular output, meaning where we send data to a device
#[derive(Queryable, Clone, Debug)]
pub struct Output {
    pub output_id: i32,

    /// What do we call this device
    pub name: String,

    /// The device this input is associated with
    pub device_id: i32,

    /// Each device can have multiple inputs and outputs, this is a device-specific index. (pin
    /// number, channel, etc)
    pub device_output_id: i32,

    /// what is the type of data that this input will produce ("Boolean", "DegC", etc)
    pub unit: String,

    /// is the circuit active_low, and hence needing flips
    pub active_low: bool,

    /// If set to an expression, the system will compute this output every state change and write it to the output
    pub automation_script: Option<String>,

    /// When was this created
    pub created_at: NaiveDateTime
}

impl Db {
    pub fn start_db(path: &std::path::Path) -> Result<Self> {
        let joined = path.join("rpi.sql3");
        let uri = joined.to_str().ok_or(Error::IoError("path not set".to_string()))?;
        Ok(Db {
            uri: uri.to_string(),
            db: get_pool(uri)
        })
    }

    pub fn devices(&self) -> Result<Vec<Device>> {
        use crate::schema::devices::dsl::*;
        let db = self.db.get()?;
        Ok(devices.load(&db)?)
    }

    pub fn inputs(&self) -> Result<Vec<Input>> {
        use crate::schema::inputs::dsl::*;
        let db = self.db.get()?;
        Ok(inputs.load(&db)?)
    }

    pub fn outputs(&self) -> Result<Vec<Output>> {
        use crate::schema::outputs::dsl::*;
        let db = self.db.get()?;
        Ok(outputs.load(&db)?)
    }
}

