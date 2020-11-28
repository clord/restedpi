use crate::error::{Error, Result};
use crypto::pbkdf2::{pbkdf2_check, pbkdf2_simple};

/**
 * allows us to change number of iterations for version
 */
fn iterations_for_version(version: usize) -> Option<u32> {
    match version {
        1 => Some(1_000_000u32),
        _ => None,
    }
}

/**
 * upon creating an account, we would use this to generate hashed password for future auth.
 */
pub fn hash(password: &str, version: usize) -> Result<String> {
    match iterations_for_version(version) {
        Some(c) => {
            let res = pbkdf2_simple(password, c)?;
            Ok(res)
        }
        None => Err(Error::NonExistant("version".to_string())),
    }
}

/**
 * check if password matches the hashed value.
 */
pub fn verify(password: &str, hashed: &str) -> Result<bool> {
    match pbkdf2_check(password, hashed) {
        Ok(b) => Ok(b),
        Err(e) => Err(Error::NonExistant(e.to_string())),
    }
}
