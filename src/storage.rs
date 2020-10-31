use crate::config;
use crate::i2c::Result;
use std::collections::HashMap;

pub struct Storage {
    db: sled::Db,
}

fn make_key(prefix: Vec<u8>, name: &str) -> Vec<u8> {
    let mut key = prefix;
    for byte in name.bytes() {
        key.push(byte)
    }
    key
}

fn make_input_key(name: &str) -> Vec<u8> {
    make_key(b"inputs/".to_vec(), name)
}

fn make_output_key(name: &str) -> Vec<u8> {
    make_key(b"outputs/".to_vec(), name)
}

fn make_device_key(name: &str) -> Vec<u8> {
    make_key(b"devices/".to_vec(), name)
}

impl Storage {
    fn all_prefix<T: serde::de::DeserializeOwned>(
        &self,
        prefix: Vec<u8>,
    ) -> Result<HashMap<String, T>> {
        let mut result: HashMap<String, T> = HashMap::new();
        let prefix_len = prefix.len();
        for item in self.db.scan_prefix(prefix) {
            let (key, value) = item?;
            let decoded: T = serde_json::from_slice(&value)?;
            result.insert(
                String::from_utf8_lossy(&key[prefix_len..]).into_owned(),
                decoded,
            );
        }
        Ok(result)
    }

    /**
     * Read all devices stored in db.
     */
    pub fn all_devices(&self) -> Result<HashMap<String, config::Device>> {
        self.all_prefix(b"devices/".to_vec())
    }

    /**
     * Read all inputs stored in db.
     */
    pub fn all_inputs(&self) -> Result<HashMap<String, config::Input>> {
        self.all_prefix(b"inputs/".to_vec())
    }

    /**
     * Read all outputs stored in db.
     */
    pub fn all_outputs(&self) -> Result<HashMap<String, config::Output>> {
        self.all_prefix(b"outputs/".to_vec())
    }

    /**
     * Will set the device with a given name to a given config.
     */
    pub fn set_device(&mut self, name: &str, device: &config::Device) -> Result<()> {
        let key = make_device_key(name);
        let encoded = serde_json::to_vec(device)?;
        self.db.insert(key, encoded)?;
        Ok(())
    }

    /**
     * Will set the device with a given name to a given config.
     */
    pub fn set_input(&mut self, name: &str, input: &config::Input) -> Result<()> {
        let key = make_input_key(name);
        let encoded = serde_json::to_vec(input)?;
        self.db.insert(key, encoded)?;
        Ok(())
    }

    /**
     * Will set the device with a given name to a given config.
     */
    pub fn set_output(&mut self, name: &str, output: &config::Output) -> Result<()> {
        let key = make_output_key(name);
        let encoded = serde_json::to_vec(output)?;
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

    pub fn remove_output(&mut self, name: &str) -> Result<()> {
        let key = make_output_key(name);
        self.db.remove(key)?;
        Ok(())
    }

    pub fn remove_input(&mut self, name: &str) -> Result<()> {
        let key = make_input_key(name);
        self.db.remove(key)?;
        Ok(())
    }

    pub fn get_input(&self, name: &str) -> Result<Option<config::Input>> {
        let key = make_input_key(name);
        if let Some(value) = self.db.get(key)? {
            let decoded = serde_json::from_slice(&value)?;
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }
    pub fn get_output(&self, name: &str) -> Result<Option<config::Output>> {
        let key = make_output_key(name);
        if let Some(value) = self.db.get(key)? {
            let decoded = serde_json::from_slice(&value)?;
            Ok(Some(decoded))
        } else {
            Ok(None)
        }
    }
}

pub fn open(path: &std::path::Path) -> sled::Result<Storage> {
    let db = sled::open(path)?;
    Ok(Storage { db })
}
