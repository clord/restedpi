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

    // Reject tokens produced under any other signing scheme before touching the payload.
    if signed_token.version != VERSION {
        return Err(SessionError::ValidationFailure);
    }

    let secret_u8 =
        hex::decode(secret).map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;

    let mut hmac = HmacSha256::new_from_slice(&secret_u8)
        .map_err(|_| SessionError::BincodeError("Length is wrong".to_string()))?;
    hmac.update(&[signed_token.version]); // version of signature
    hmac.update(&signed_token.payload); // bytes of payload
    hmac.verify_slice(&signed_token.signature)
        .map_err(|_| SessionError::ValidationFailure)?;

    // Only deserialize the payload once the signature has been verified.
    bincode::deserialize(&signed_token.payload)
        .map_err(|x| SessionError::BincodeError(format!("bincode_deser_tok: {}", x)))
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

    #[test]
    fn tampered_payload_fails() -> Result<(), SessionError> {
        let valid = SessionToken {
            session_version: 1,
            id: 1212,
        };
        let secret = "0123456789abcdef";
        let token = make_token(valid, secret)?;

        // Decode the wrapper, flip a bit in the payload, and re-encode.
        let raw = hex::decode(&token)
            .map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;
        let mut signed: SignedToken = bincode::deserialize(&raw)
            .map_err(|x| SessionError::BincodeError(format!("bincode_deser_wrap: {}", x)))?;
        match signed.payload.first_mut() {
            Some(byte) => *byte ^= 0xff,
            None => panic!("payload must not be empty"),
        }
        let tampered = hex::encode(
            bincode::serialize(&signed)
                .map_err(|x| SessionError::BincodeError(format!("bincode_ser: {}", x)))?,
        );

        assert_eq!(
            validate_token::<SessionToken>(&tampered, secret),
            Err(SessionError::ValidationFailure)
        );
        Ok(())
    }

    #[test]
    fn wrong_version_fails() -> Result<(), SessionError> {
        let valid = SessionToken {
            session_version: 1,
            id: 1212,
        };
        let secret = "0123456789abcdef";
        let payload = bincode::serialize(&valid)
            .map_err(|x| SessionError::BincodeError(format!("bincode_ser: {}", x)))?;
        let secret_u8 = hex::decode(secret)
            .map_err(|x| SessionError::HexcodeError(format!("hexcode: {}", x)))?;

        // Craft a token whose signature is valid for its claimed version, but
        // whose version is not the one this build accepts.
        let bogus_version = VERSION.wrapping_add(1);
        let mut hmac = HmacSha256::new_from_slice(&secret_u8)
            .map_err(|_| SessionError::BincodeError("Length is wrong".to_string()))?;
        hmac.update(&[bogus_version]);
        hmac.update(&payload);
        let signed = SignedToken {
            version: bogus_version,
            signature: hmac.finalize().into_bytes().to_vec(),
            payload,
        };
        let token = hex::encode(
            bincode::serialize(&signed)
                .map_err(|x| SessionError::BincodeError(format!("bincode_ser: {}", x)))?,
        );

        assert_eq!(
            validate_token::<SessionToken>(&token, secret),
            Err(SessionError::ValidationFailure)
        );
        Ok(())
    }

    #[test]
    fn valid_round_trip() -> Result<(), SessionError> {
        let valid = SessionToken {
            session_version: 3,
            id: 42,
        };
        let secret = "deadbeefdeadbeef";
        let token = make_token(valid.clone(), secret)?;
        assert_eq!(validate_token::<SessionToken>(&token, secret)?, valid);
        Ok(())
    }
}
