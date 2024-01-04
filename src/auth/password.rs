use crate::error::{Error, Result};
use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

/**
 * upon creating an account, we would use this to generate hashed password for future auth.
 */
pub fn hash(password: &str) -> Result<String> {
    match SaltString::from_b64("185615397a8cca229fecd23b6b523eb3083069c2") {
        Ok(salt) => match Pbkdf2.hash_password(password.as_bytes(), &salt) {
            Ok(res) => Ok(res.to_string()),
            Err(e) => Err(Error::PbkError(e.to_string())),
        },
        Err(e) => Err(Error::PbkError(e.to_string())),
    }
}

/**
 * check if password matches the hashed value.
 */
pub fn verify(password: &str, hashed: &str) -> Result<()> {
    match PasswordHash::new(hashed) {
        Ok(parsed) => match Pbkdf2.verify_password(password.as_bytes(), &parsed) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::PbkError(e.to_string())),
        },
        Err(e) => Err(Error::PbkError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        assert_eq!(verify("f00", &hash("f00")?), Ok(()));
        assert_eq!(
            verify("f00a", &hash("f00")?),
            Err(Error::PbkError("Something".to_string()))
        );
        assert_eq!(verify("", &hash("")?), Ok(()));
        Ok(())
    }
}
