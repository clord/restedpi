use crate::error::{Error, Result};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SessionToken {
    /**
     * format of unsigned token
     */
    pub session_version: u8,

    pub id: usize,
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

static VERSION: u8 = 1u8;

/**
 * make a signed token for use as a cookie, like a jwt but not sucky.
 * v1:
 * - always hmac sha256 signature
 * - payload is binary serialization of token
 *
 * output is hex-encoded
 */
pub fn make_token<T: serde::Serialize>(token: T, secret: &str) -> Result<String> {
    let payload = bincode::serialize(&token)
        .map_err(|x| Error::EncodingError(format!("bincode_ser: {}", x)))?;
    let secret_u8 = hex::decode(secret)?;
    let mut hmac = Hmac::new(crypto::sha2::Sha256::new(), &secret_u8);
    hmac.input(&[VERSION]); // version of signature
    hmac.input(&payload); // bytes of payload
    let mut signature: Vec<u8> = Vec::new();
    signature.resize(hmac.output_bytes(), 0);
    hmac.raw_result(&mut signature);

    let signed = SignedToken {
        version: VERSION,
        signature,
        payload,
    };

    let raw = bincode::serialize(&signed)
        .map_err(|x| Error::EncodingError(format!("bincode_ser: {}", x)))?;

    Ok(hex::encode(raw))
}

/**
 * given a serialized token and the secret, will determine if the token is valid according to
 * secret.
 */
pub fn validate_token<T: serde::de::DeserializeOwned>(token: &str, secret: &str) -> Result<T> {
    let raw = hex::decode(token)?;
    let signed_token: SignedToken = bincode::deserialize(&raw)
        .map_err(|x| Error::EncodingError(format!("bincode_deser: {}", x)))?;
    let decoded: T = bincode::deserialize(&signed_token.payload)
        .map_err(|x| Error::EncodingError(format!("bincode_deser: {}", x)))?;
    let secret_u8 = hex::decode(secret)?;
    let mut hmac = Hmac::new(crypto::sha2::Sha256::new(), &secret_u8);

    hmac.input(&[VERSION]); // version of signature
    hmac.input(&signed_token.payload); // bytes of payload

    if hmac
        .result()
        .eq(&crypto::mac::MacResult::new(&signed_token.signature))
    {
        Ok(decoded)
    } else {
        Err(Error::EncodingError(
            "Failed to verify signature".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<()> {
        let valid = SessionToken {
            session_version: 1,
            id: 1212,
        };
        let secret = "0123456789abcdef";
        let token = make_token(valid.clone(), secret)?;
        assert_eq!(validate_token::<SessionToken>(&token, secret)?, valid);
        assert_eq!(
            validate_token::<SessionToken>(&token, "123123123123"),
            Err(Error::EncodingError(
                "Failed to verify signature".to_string()
            ))
        );
        Ok(())
    }
}
