use crate::app::channel::AppChannel;
use crate::auth::{password, secret, token};
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
/// If invalid for any reason (crypto, expired, missing app secret, etc) then it will result in an error.
impl FromStr for WebSession {
    type Err = token::SessionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let secret = secret::get().ok_or(token::SessionError::MissingSecret)?;
        let start = SystemTime::now();
        let now_timestamp = start
            .duration_since(UNIX_EPOCH)
            .map_err(|_| token::SessionError::Expired)?;
        let res = token::validate_token::<WebSession>(s, secret)?;
        if res.expires < now_timestamp.as_secs() {
            Err(token::SessionError::Expired)
        } else {
            Ok(res)
        }
    }
}

/// How long an issued session token stays valid: 90 days, in seconds.
const TOKEN_TTL_SECONDS: u64 = 60 * 60 * 24 * 90;

pub async fn authenticate(ctx: &AppContext, user: &str, pw: &str) -> Result<String> {
    let secret = secret::get().ok_or_else(|| {
        Error::Config(
            "application secret not configured (set app_secret_path or APP_SECRET)".to_string(),
        )
    })?;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::Config("System time before UNIX epoch".to_string()))?;

    let expires = since_the_epoch
        .checked_add(Duration::new(TOKEN_TTL_SECONDS, 0))
        .ok_or_else(|| Error::Config("Token expiration overflow".to_string()))?
        .as_secs();

    match ctx.channel().hash_for(user) {
        Some(user_hash) => match password::verify(pw, user_hash) {
            Ok(()) => match token::make_token(
                WebSession {
                    user: user.to_string(),
                    expires,
                },
                secret,
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

    /// Hex-encoded secret used by the session tests.
    const TEST_SECRET: &str = "0123456789abcdef0123456789abcdef";

    /// Initialize the process-global secret store with the test secret.
    /// Every test in this module uses the same value, so losing the
    /// `AlreadyInitialized` race to a sibling test is fine.
    fn ensure_app_secret() {
        let _ = crate::auth::secret::init(TEST_SECRET.to_string());
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
