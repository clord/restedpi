use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct UnsignedToken {
    /**
     * format of unsigned token
     */
    pub version: u8,

    pub id: usize
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
    pub payload: Vec<u8>
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
pub fn make_token(token: UnsignedToken, secret: &str) -> Result<String> {
    let payload = bincode::serialize(token);
    let secret_u8 = hex::decode(secret);
    let hmac = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), secret_u8);
    hmac.input(VERSION); // version of signature
    hmac.input(payload); // bytes of payload
    let mut signature: Vec<u8> = Vec::new();
    hmac.raw_result(signature);

    let raw =  bincode::serialize(SignedToken {
        version: VERSION,
        signature,
        payload
    })?;

    hex::encode(raw)
}

/**
 * given a serialized token and the secret, will determine if the token is valid according to
 * secret.
 */
pub fn validate_token(token: &str, secret: &str) -> bool {
    let raw = hex::decode(token);
    let signed_token: SignedToken = bincode::deserialize(raw);
    let secret_u8 = hex::decode(secret);
    let hmac = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), secret_u8);
    hmac.input(VERSION); // version of signature
    hmac.input(payload); // bytes of payload
    hmac.result().eq(crypto::mac::MacResult::new(signed_token.signuatre))
}
