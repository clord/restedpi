pub mod models;

use crate::app::AppID;
use crate::error::{Error, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use std::path::Path;
use tracing::info;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// SQL statements for bootstrapping a new database (executed in order)
const SCHEMA_STATEMENTS: &[&str] = &[
    "PRAGMA foreign_keys = ON",
    "CREATE TABLE IF NOT EXISTS devices(
        name TEXT NOT NULL PRIMARY KEY,
        model TEXT NOT NULL,
        notes TEXT NOT NULL,
        disabled BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    )",
    "CREATE TABLE IF NOT EXISTS inputs(
        name TEXT NOT NULL PRIMARY KEY,
        device_id TEXT NOT NULL,
        device_input_id INT NOT NULL,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (device_id) REFERENCES devices(name) ON DELETE CASCADE
    )",
    "CREATE TABLE IF NOT EXISTS outputs(
        name TEXT NOT NULL PRIMARY KEY,
        device_id TEXT NOT NULL,
        device_output_id INT NOT NULL,
        active_low BOOLEAN NOT NULL DEFAULT FALSE,
        automation_script TEXT,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (device_id) REFERENCES devices(name) ON DELETE CASCADE
    )",
];

fn get_pool(db_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    Pool::new(manager).map_err(|e| Error::DbError(format!("Failed to create DB pool: {}", e)))
}

pub struct Db {
    db: DbPool,
}

impl Db {
    /// Initialize the database, creating it if necessary.
    ///
    /// The database file will be created at `{path}/rpi.sql3`.
    /// If the file doesn't exist or is empty, the schema will be bootstrapped.
    pub fn start_db(path: &Path) -> Result<Self> {
        // Ensure the directory exists
        if !path.exists() {
            std::fs::create_dir_all(path).map_err(|e| {
                Error::DbError(format!(
                    "Failed to create database directory {:?}: {}. \
                     Set db_path in config.toml or use --config-file to specify location.",
                    path, e
                ))
            })?;
            info!("Created database directory: {:?}", path);
        }

        let db_file = path.join("rpi.sql3");
        let db_uri = db_file.to_str().ok_or_else(|| {
            Error::IoError(format!(
                "Database path {:?} contains invalid UTF-8",
                db_file
            ))
        })?;

        let needs_bootstrap = !db_file.exists();

        // Create the connection pool
        let pool = get_pool(db_uri).map_err(|e| {
            Error::DbError(format!(
                "Failed to open database at {:?}: {}. \
                 Check that the path is writable and has sufficient disk space.",
                db_file, e
            ))
        })?;

        // Bootstrap schema if this is a new database
        if needs_bootstrap {
            info!("Bootstrapping new database at {:?}", db_file);
            let mut conn = pool.get().map_err(|e| {
                Error::DbError(format!("Failed to get connection for bootstrap: {}", e))
            })?;

            for statement in SCHEMA_STATEMENTS {
                diesel::sql_query(*statement).execute(&mut conn).map_err(|e| {
                    Error::DbError(format!("Failed to execute schema statement: {}", e))
                })?;
            }

            info!("Database schema created successfully");
        } else {
            info!("Using existing database at {:?}", db_file);
        }

        Ok(Db { db: pool })
    }

    pub fn add_device(&self, new_device: &models::NewDevice) -> Result<models::Device> {
        use crate::schema::devices::dsl::*;
        use crate::schema::devices::table;
        let mut db = self.db.get()?;
        let res = diesel::insert_into(table)
            .values(new_device)
            .execute(&mut db)?;
        info!("Added {} rows to device table", res);
        let r: models::Device = devices.find(&new_device.name).first(&mut db)?;
        Ok(r)
    }

    pub fn remove_device(&self, device_id: &AppID) -> Result<()> {
        use crate::schema::{devices, inputs, outputs};
        let mut db = self.db.get()?;
        db.transaction(|conn| {
            diesel::delete(inputs::dsl::inputs.filter(inputs::dsl::device_id.eq(device_id)))
                .execute(conn)?;
            diesel::delete(outputs::dsl::outputs.filter(outputs::dsl::device_id.eq(device_id)))
                .execute(conn)?;
            diesel::delete(devices::dsl::devices.filter(devices::dsl::name.eq(device_id)))
                .execute(conn)?;
            Ok(())
        })
    }

    pub fn device(&self, did: &AppID) -> Result<models::Device> {
        use crate::schema::devices::dsl::*;
        let mut db = self.db.get()?;
        Ok(devices.filter(name.eq(did)).first(&mut db)?)
    }

    pub fn devices(&self) -> Result<Vec<models::Device>> {
        use crate::schema::devices::dsl::*;
        let mut db = self.db.get()?;
        Ok(devices.load(&mut db)?)
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
        let mut db = self.db.get()?;
        let out = inputs::dsl::inputs.load(&mut db)?;
        Ok(out)
    }

    pub fn outputs(&self) -> Result<Vec<models::Output>> {
        use crate::schema::outputs;
        let mut db = self.db.get()?;
        let out = outputs::dsl::outputs.load(&mut db)?;
        Ok(out)
    }

    pub fn remove_output(&self, id: &AppID) -> Result<()> {
        use crate::schema::outputs::dsl::*;
        let mut db = self.db.get()?;
        diesel::delete(outputs.filter(name.eq(id))).execute(&mut db)?;
        Ok(())
    }

    pub fn output(&self, oid: &AppID) -> Result<models::Output> {
        use crate::schema::outputs;
        let mut db = self.db.get()?;
        let out = outputs::dsl::outputs
            .filter(outputs::name.eq(oid))
            .first(&mut db)?;
        Ok(out)
    }

    pub fn input(&self, iid: &AppID) -> Result<models::Input> {
        use crate::schema::inputs;
        let mut db = self.db.get()?;
        let inp = inputs::dsl::inputs
            .filter(inputs::name.eq(iid))
            .first(&mut db)?;
        Ok(inp)
    }

    pub fn remove_input(&self, id: &AppID) -> Result<()> {
        use crate::schema::inputs::dsl::*;
        let mut db = self.db.get()?;
        diesel::delete(inputs.filter(name.eq(id))).execute(&mut db)?;
        Ok(())
    }

    pub fn add_input(&self, new_input: &models::NewInput) -> Result<models::Input> {
        use crate::schema::inputs::dsl::*;
        use crate::schema::inputs::table;
        let mut db = self.db.get()?;
        let res = diesel::insert_into(table)
            .values(new_input)
            .execute(&mut db)?;
        info!("Added {} rows to input table", res);
        let r: models::Input = inputs.find(&new_input.name).first(&mut db)?;
        Ok(r)
    }

    pub fn update_output(
        &self,
        old_output_id: &AppID,
        fields: &models::UpdateOutput,
    ) -> Result<models::Output> {
        use crate::schema::outputs::dsl::*;
        use crate::schema::outputs::table;
        let mut db = self.db.get()?;

        if let models::UpdateOutput {
            device_output_id: Some(f),
            ..
        } = fields
        {
            let ex = diesel::update(table)
                .filter(name.eq(old_output_id))
                .set(device_output_id.eq(f));
            let res = ex.execute(&mut db)?;
            info!("updated {} rows of output table", res);
        }

        if let models::UpdateOutput {
            active_low: Some(f),
            ..
        } = fields
        {
            let ex = diesel::update(table)
                .filter(name.eq(old_output_id))
                .set(active_low.eq(*f));
            let res = ex.execute(&mut db)?;
            info!("updated {} rows of output table", res);
        }

        if let models::UpdateOutput {
            automation_script: Some(f),
            ..
        } = fields
        {
            let ex = diesel::update(table)
                .filter(name.eq(old_output_id))
                .set(automation_script.eq(f));
            let res = ex.execute(&mut db)?;
            info!("updated {} rows of output table", res);
        }

        let r: models::Output = outputs.find(old_output_id).first(&mut db)?;
        Ok(r)
    }

    pub fn add_output(&self, new_output: &models::NewOutput) -> Result<models::Output> {
        use crate::schema::outputs::dsl::*;
        use crate::schema::outputs::table;
        let mut db = self.db.get()?;
        let res = diesel::insert_into(table)
            .values(new_output)
            .execute(&mut db)?;
        info!("Added {} rows to output table", res);
        let r: models::Output = outputs.find(&new_output.name).first(&mut db)?;
        Ok(r)
    }

    pub fn inputs_for_device(&self, d_id: &AppID) -> Result<Vec<models::Input>> {
        use crate::schema::inputs::dsl::*;
        let mut db = self.db.get()?;
        Ok(inputs.filter(device_id.eq(d_id)).load(&mut db)?)
    }

    pub fn outputs_for_device(&self, d_id: &AppID) -> Result<Vec<models::Output>> {
        use crate::schema::outputs::dsl::*;
        let mut db = self.db.get()?;
        Ok(outputs.filter(device_id.eq(d_id)).load(&mut db)?)
    }
}
