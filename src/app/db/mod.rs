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

/// Per-connection SQLite setup for every connection handed out by the pool.
///
/// SQLite enforces `PRAGMA foreign_keys` per connection, so it must be enabled
/// on each pooled connection or `ON DELETE CASCADE` is silently ignored.
/// `busy_timeout` makes concurrent writers wait instead of failing immediately.
#[derive(Debug, Clone, Copy)]
struct SqliteConnectionCustomizer;

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for SqliteConnectionCustomizer
{
    fn on_acquire(
        &self,
        conn: &mut SqliteConnection,
    ) -> std::result::Result<(), diesel::r2d2::Error> {
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;
        diesel::sql_query("PRAGMA busy_timeout = 5000;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;
        Ok(())
    }
}

fn get_pool(db_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    Pool::builder()
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .map_err(|e| Error::DbError(format!("Failed to create DB pool: {}", e)))
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
                diesel::sql_query(*statement)
                    .execute(&mut conn)
                    .map_err(|e| {
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

        // Apply all requested field updates atomically: either every provided
        // field is updated, or none are.
        db.transaction(|conn| {
            if let models::UpdateOutput {
                device_output_id: Some(f),
                ..
            } = fields
            {
                let ex = diesel::update(table)
                    .filter(name.eq(old_output_id))
                    .set(device_output_id.eq(f));
                let res = ex.execute(conn)?;
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
                let res = ex.execute(conn)?;
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
                let res = ex.execute(conn)?;
                info!("updated {} rows of output table", res);
            }

            let r: models::Output = outputs.find(old_output_id).first(conn)?;
            Ok(r)
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::device::{Directions, MCP23017, Type};

    /// Create a fresh database in a unique temporary directory
    fn fresh_db(tag: &str) -> Db {
        let dir =
            std::env::temp_dir().join(format!("restedpi-db-test-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Db::start_db(&dir).expect("failed to create test database")
    }

    fn test_device_type() -> Type {
        Type::MCP23017(MCP23017 {
            address: 0x20,
            bank_a: Directions::new(),
            bank_b: Directions::new(),
        })
    }

    fn add_test_device(db: &Db, name: &str) -> models::Device {
        let new_device = models::NewDevice::new(
            test_device_type(),
            name.to_string(),
            "test device".to_string(),
            None,
        );
        db.add_device(&new_device).expect("failed to add device")
    }

    #[test]
    fn foreign_key_cascade_deletes_children_on_raw_device_delete() {
        let db = fresh_db("fk-cascade");
        add_test_device(&db, "dev1");

        db.add_input(&models::NewInput::new(
            "in1".to_string(),
            "dev1".to_string(),
            0,
        ))
        .expect("failed to add input");
        db.add_output(&models::NewOutput::new(
            "out1".to_string(),
            "dev1".to_string(),
            1,
            false,
            None,
        ))
        .expect("failed to add output");

        assert_eq!(db.inputs().unwrap().len(), 1);
        assert_eq!(db.outputs().unwrap().len(), 1);

        // Delete the device row directly via SQL, bypassing Db::remove_device,
        // so only SQLite's ON DELETE CASCADE can clean up the children.
        let mut conn = db.db.get().unwrap();
        diesel::sql_query("DELETE FROM devices WHERE name = 'dev1'")
            .execute(&mut conn)
            .expect("raw delete failed");
        drop(conn);

        assert!(
            db.inputs().unwrap().is_empty(),
            "FK cascade should remove child inputs"
        );
        assert!(
            db.outputs().unwrap().is_empty(),
            "FK cascade should remove child outputs"
        );
    }

    #[test]
    fn update_output_applies_all_fields() {
        let db = fresh_db("update-output");
        add_test_device(&db, "dev1");

        db.add_output(&models::NewOutput::new(
            "out1".to_string(),
            "dev1".to_string(),
            0,
            false,
            None,
        ))
        .expect("failed to add output");

        let updated = db
            .update_output(
                &"out1".to_string(),
                &models::UpdateOutput {
                    device_output_id: Some(7),
                    active_low: Some(true),
                    automation_script: Some(Some("true".to_string())),
                },
            )
            .expect("update_output failed");

        assert_eq!(updated.device_output_id, 7);
        assert!(updated.active_low);
        assert_eq!(updated.automation_script, Some("true".to_string()));

        // Clearing the automation script via Some(None) should also persist
        let cleared = db
            .update_output(
                &"out1".to_string(),
                &models::UpdateOutput {
                    device_output_id: None,
                    active_low: None,
                    automation_script: Some(None),
                },
            )
            .expect("update_output failed");

        assert_eq!(cleared.device_output_id, 7);
        assert!(cleared.active_low);
        assert_eq!(cleared.automation_script, None);
    }
}
