use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(PartialEq, Debug)]
pub enum SessionError {
    BincodeError(String),
    HexcodeError(String),
    ValidationFailure,
    Expired,
    MissingSecret,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SignedToken {
    /**
     * version of signing to use (1 for sha256 hmac)
     */
    pub version: u8,

    /**
     * sha256 hmac signature of the payload
     */
    pub signature: Vec<u8>,

    /**
     * Exact bytes that were signed (deserializable to UnsignedToken)
     */
    pub payload: Vec<u8>,
}

/**
 * If we increment this then old tokens will not be valid.
 */
static VERSION: u8 = 1u8;

type HmacSha256 = Hmac<Sha256>;

/**
 * make a signed token for use as a cookie, like a jwt but not sucky.
 * v1:
 * - always hmac sha256 signature
 * - payload is binary serialization of token
 *
 * output is hex-encoded
 */
pub fn make_token<T: serde::Serialize>(token: T, secret: &str) -> Result<String, SessionError> {
    let payload = bincode::serialize(&token)
        .map_err(|x| SessionError::BincodeError(format!("bincode_ser: {}", x)))?;
    let secret_u8 =
        hex::decode(secret).map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;
    if let Ok(mut hmac) = HmacSha256::new_from_slice(&secret_u8) {
        hmac.update(&[VERSION]); // version of signature
        hmac.update(&payload); // bytes of payload
        let signature = hmac.finalize().into_bytes();

        let signed = SignedToken {
            version: VERSION,
            signature: signature.to_vec(),
            payload,
        };

        let raw = bincode::serialize(&signed)
            .map_err(|x| SessionError::BincodeError(format!("bincode_ser: {}", x)))?;

        Ok(hex::encode(raw))
    } else {
        Err(SessionError::BincodeError("Length is wrong".to_string()))
    }
}

/**
 * given a serialized token and the secret, will determine if the token is valid according to
 * secret.
 */
pub fn validate_token<T: serde::de::DeserializeOwned>(
    token: &str,
    secret: &str,
) -> Result<T, SessionError> {
    let raw =
        hex::decode(token).map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;
    let signed_token: SignedToken = bincode::deserialize(&raw)
        .map_err(|x| SessionError::BincodeError(format!("bincode_deser_wrap: {}", x)))?;
    let decoded: T = bincode::deserialize(&signed_token.payload)
        .map_err(|x| SessionError::BincodeError(format!("bincode_deser_tok: {}", x)))?;
    let secret_u8 =
        hex::decode(secret).map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;

    if let Ok(mut hmac) = HmacSha256::new_from_slice(&secret_u8) {
        hmac.update(&[VERSION]); // version of signature
        hmac.update(&signed_token.payload); // bytes of payload
        if let Ok(()) = hmac.verify_slice(&signed_token.signature) {
            Ok(decoded)
        } else {
            Err(SessionError::ValidationFailure)
        }
    } else {
        Err(SessionError::BincodeError("Length is wrong".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
    pub struct SessionToken {
        /**
         * format of unsigned token
         */
        pub session_version: u8,

        pub id: usize,
    }

    #[test]
    fn basic() -> Result<(), SessionError> {
        let valid = SessionToken {
            session_version: 1,
            id: 1212,
        };
        let secret = "0123456789abcdef";
        let token = make_token(valid.clone(), secret)?;
        assert_eq!(validate_token::<SessionToken>(&token, secret)?, valid);
        assert_eq!(
            validate_token::<SessionToken>(&token, "123123123123"),
            Err(SessionError::ValidationFailure)
        );
        Ok(())
    }
}
