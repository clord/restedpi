use crate::error::{Error, Result};
use argon2::Argon2;
use password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

/**
 * upon creating an account, we would use this to generate hashed password for future auth.
 */
pub fn hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(res) => Ok(res.to_string()),
        Err(e) => Err(Error::PbkError(format!("failed hashing password: {}", e))),
    }
}

/**
 * check if password matches the hashed value.
 */
pub fn verify(password: &str, hashed: &str) -> Result<()> {
    let argon2 = Argon2::default();
    match PasswordHash::new(hashed) {
        Ok(hash) => match argon2.verify_password(password.as_bytes(), &hash) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::PbkError(format!("verify failed: {}", e))),
        },
        Err(e) => Err(Error::PbkError(format!("decode of password failed: {}", e))),
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
            Err(Error::PbkError(
                "verify failed: invalid password".to_string()
            ))
        );
        assert_eq!(verify("", &hash("")?), Ok(()));
        Ok(())
    }
}
