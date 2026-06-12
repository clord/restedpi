use crate::app::channel::AppChannel;
use crate::auth::{password, token};
use crate::error::{Error, Result};
use crate::webapp::SharedAppState;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{error, warn};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WebSession {
    pub user: String,
    pub expires: u64,
}

/// All information relating to current state of the app and authentication
pub struct AppContext {
    main: SharedAppState,
    pub session: Option<WebSession>,
}

impl AppContext {
    pub fn new(main: SharedAppState, session: Option<WebSession>) -> Self {
        Self { main, session }
    }

    pub fn channel(&self) -> &AppChannel {
        &self.main
    }
}

impl juniper::Context for AppContext {}

/// We can parse sessions from strings.
/// If invalid for any reason (crypto, expired, missing APP_SECRET, etc) then it will result in an error.
impl FromStr for WebSession {
    type Err = token::SessionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let secret = std::env::var("APP_SECRET").map_err(|_| token::SessionError::MissingSecret)?;
        let start = SystemTime::now();
        let now_timestamp = start
            .duration_since(UNIX_EPOCH)
            .map_err(|_| token::SessionError::Expired)?;
        let res = token::validate_token::<WebSession>(s, &secret)?;
        if res.expires < now_timestamp.as_secs() {
            Err(token::SessionError::Expired)
        } else {
            Ok(res)
        }
    }
}

const TOKEN_DURATION: u64 = 60 * 60 * 24 * 660;

pub async fn authenticate(ctx: &AppContext, user: &str, pw: &str) -> Result<String> {
    let secret = std::env::var("APP_SECRET")
        .map_err(|_| Error::Config("APP_SECRET environment variable not set".to_string()))?;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::Config("System time before UNIX epoch".to_string()))?;

    let expires = since_the_epoch
        .checked_add(Duration::new(TOKEN_DURATION, 0))
        .ok_or_else(|| Error::Config("Token expiration overflow".to_string()))?
        .as_secs();

    match ctx.channel().hash_for(user) {
        Some(user_hash) => match password::verify(pw, user_hash) {
            Ok(()) => match token::make_token(
                WebSession {
                    user: user.to_string(),
                    expires,
                },
                &secret,
            ) {
                Ok(token) => Ok(token),
                Err(e) => {
                    error!("Error generating token: {:?}", e);
                    Err(Error::TokenIssue)
                }
            },
            Err(e) => {
                warn!("Password verification failed: {}", e);
                Err(Error::PasswordIssue)
            }
        },
        None => Err(Error::UserNotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    /// Hex-encoded secret used by the session tests.
    const TEST_SECRET: &str = "0123456789abcdef0123456789abcdef";

    static INIT_SECRET: Once = Once::new();

    /// Set APP_SECRET exactly once for the whole test process.
    /// All tests in this module share the same value, so concurrent readers
    /// always observe a consistent secret.
    fn ensure_app_secret() {
        INIT_SECRET.call_once(|| {
            // SAFETY: called exactly once; no other test in this crate mutates
            // the process environment.
            unsafe { std::env::set_var("APP_SECRET", TEST_SECRET) };
        });
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_secs()
    }

    #[test]
    fn expired_session_token_is_rejected() {
        ensure_app_secret();
        let expired = WebSession {
            user: "tester".to_string(),
            expires: now_secs() - 3600,
        };
        let token = token::make_token(expired, TEST_SECRET).expect("make token");
        let res = WebSession::from_str(&token);
        assert!(
            matches!(res, Err(token::SessionError::Expired)),
            "expected Expired, got {:?}",
            res
        );
    }

    #[test]
    fn unexpired_session_token_is_accepted() {
        ensure_app_secret();
        let valid = WebSession {
            user: "tester".to_string(),
            expires: now_secs() + 3600,
        };
        let token = token::make_token(valid, TEST_SECRET).expect("make token");
        let session = WebSession::from_str(&token).expect("valid session");
        assert_eq!(session.user, "tester");
    }

    #[test]
    fn garbage_token_is_rejected() {
        ensure_app_secret();
        let res = WebSession::from_str("not-a-token");
        assert!(res.is_err(), "garbage tokens must not produce a session");
    }
}
