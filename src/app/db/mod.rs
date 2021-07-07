pub mod models;

use crate::app::AppID;
use crate::error::{Error, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use tracing::info;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

fn get_pool(db_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    Pool::new(manager).expect("Failed to create DB Pool")
}

pub struct Db {
    db: DbPool,
}

impl Db {
    pub fn start_db(path: &std::path::Path) -> Result<Self> {
        let joined = path.join("rpi.sql3");
        let uri = joined
            .to_str()
            .ok_or(Error::IoError("path not set".to_string()))?;
        Ok(Db { db: get_pool(uri) })
    }

    pub fn add_device(&self, new_device: &models::NewDevice) -> Result<models::Device> {
        use crate::schema::devices::dsl::*;
        use crate::schema::devices::table;
        let db = self.db.get()?;
        let res = diesel::insert_into(table).values(new_device).execute(&db)?;
        info!("Added {} rows to device table", res);
        let r: models::Device = devices.find(&new_device.name).first(&db)?;
        Ok(r)
    }

    pub fn remove_device(&self, device_id: &AppID) -> Result<()> {
        use crate::schema::{devices, inputs, outputs};
        let db = self.db.get()?;
        db.transaction(|| {
            diesel::delete(inputs::dsl::inputs.filter(inputs::dsl::device_id.eq(device_id))).execute(&db)?;
            diesel::delete(outputs::dsl::outputs.filter(outputs::dsl::device_id.eq(device_id))).execute(&db)?;
            diesel::delete(devices::dsl::devices.filter(devices::dsl::name.eq(device_id))).execute(&db)?;
            Ok(())
        })
    }

    pub fn device(&self, did: &AppID) -> Result<models::Device> {
        use crate::schema::devices::dsl::*;
        let db = self.db.get()?;
        Ok(devices.filter(name.eq(did)).first(&db)?)
    }

    pub fn devices(&self) -> Result<Vec<models::Device>> {
        use crate::schema::devices::dsl::*;
        let db = self.db.get()?;
        Ok(devices.load(&db)?)
    }

    pub fn app_devices(&self) -> Result<Vec<crate::app::device::Device>> {
        let devs = self.devices()?;
        Ok(devs
            .iter()
            .map(|d| crate::app::device::Device {
                db_device: d.clone(),
            })
            .collect())
    }

    pub fn inputs(&self) -> Result<Vec<models::Input>> {
        use crate::schema::inputs;
        let db = self.db.get()?;
        let out = inputs::dsl::inputs.load(&db)?;
        Ok(out)
    }

    pub fn outputs(&self) -> Result<Vec<models::Output>> {
        use crate::schema::outputs;
        let db = self.db.get()?;
        let out = outputs::dsl::outputs.load(&db)?;
        Ok(out)
    }

    pub fn remove_output(&self, id: &AppID) -> Result<()> {
        use crate::schema::outputs::dsl::*;
        let db = self.db.get()?;
        diesel::delete(outputs.filter(name.eq(id))).execute(&db)?;
        Ok(())
    }

    pub fn output(&self, oid: &AppID) -> Result<models::Output> {
        use crate::schema::outputs;
        let db = self.db.get()?;
        let out = outputs::dsl::outputs
            .filter(outputs::name.eq(oid))
            .first(&db)?;
        Ok(out)
    }

    pub fn input(&self, iid: &AppID) -> Result<models::Input> {
        use crate::schema::inputs;
        let db = self.db.get()?;
        let inp = inputs::dsl::inputs
            .filter(inputs::name.eq(iid))
            .first(&db)?;
        Ok(inp)
    }

    pub fn remove_input(&self, id: &AppID) -> Result<()> {
        use crate::schema::inputs::dsl::*;
        let db = self.db.get()?;
        diesel::delete(inputs.filter(name.eq(id))).execute(&db)?;
        Ok(())
    }

    pub fn add_input(&self, new_input: &models::NewInput) -> Result<models::Input> {
        use crate::schema::inputs::dsl::*;
        use crate::schema::inputs::table;
        let db = self.db.get()?;
        let res = diesel::insert_into(table).values(new_input).execute(&db)?;
        info!("Added {} rows to input table", res);
        let r: models::Input = inputs.find(&new_input.name).first(&db)?;
        Ok(r)
    }

    pub fn add_output(&self, new_output: &models::NewOutput) -> Result<models::Output> {
        use crate::schema::outputs::dsl::*;
        use crate::schema::outputs::table;
        let db = self.db.get()?;
        let res = diesel::insert_into(table).values(new_output).execute(&db)?;
        info!("Added {} rows to output table", res);
        let r: models::Output = outputs.find(&new_output.name).first(&db)?;
        Ok(r)
    }

    pub fn inputs_for_device(&self, d_id: &AppID) -> Result<Vec<models::Input>> {
        use crate::schema::inputs::dsl::*;
        let db = self.db.get()?;
        Ok(inputs.filter(device_id.eq(d_id)).load(&db)?)
    }

    pub fn outputs_for_device(&self, d_id: &AppID) -> Result<Vec<models::Output>> {
        use crate::schema::outputs::dsl::*;
        let db = self.db.get()?;
        Ok(outputs.filter(device_id.eq(d_id)).load(&db)?)
    }
}
