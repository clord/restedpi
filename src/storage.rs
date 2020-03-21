use crate::config;
use crate::i2c::Result;
use std::collections::HashMap;

pub struct Storage {
    db: sled::Db,
}

fn make_device_key(name: &str) -> Vec<u8> {
    let mut key = b"devices/".to_vec();
    for byte in name.bytes() {
        key.push(byte)
    }
    key
}

impl Storage {
    /**
     * Read all devices stored in db.
     */
    pub fn read_devices(&self) -> Result<HashMap<String, config::Device>> {
        let mut result: HashMap<String, config::Device> = HashMap::new();
        for item in self.db.scan_prefix(b"devices/") {
            let (key, value) = item?;
            let decoded = bincode::deserialize(&value)?;
            result.insert(String::from_utf8_lossy(&key[8..]).into_owned(), decoded);
        }
        Ok(result)
    }

    /**
     * Will set the device with a given name to a given config.
     */
    pub fn set_device(&mut self, name: &str, device: &config::Device) -> Result<()> {
        let key = make_device_key(name);
        let encoded = bincode::serialize(device)?;
        self.db.insert(key, encoded)?;
        Ok(())
    }

    /**
     * remove the device, regardless of whether it exists.
     */
    pub fn remove_device(&mut self, name: &str) -> Result<()> {
        let key = make_device_key(name);
        self.db.remove(key)?;
        Ok(())
    }
}

pub fn open(path: &std::path::Path) -> sled::Result<Storage> {
    let db = sled::open(path)?;
    Ok(Storage { db })
}
