use crate::app::channel::AppChannel;
use crate::auth::{password, token};
use crate::error::{Error, Result};
use crate::webapp::SharedAppState;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    pub fn channel(&self) -> AppChannel {
        self.main.clone().lock().expect("locked state").clone()
    }
}

/// We can parse sessions from strings.
/// If invalid for any reason (crypto, expired, etc) then it will result in an error.
impl FromStr for WebSession {
    type Err = token::SessionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let secret =
            std::env::var("APP_SECRET").expect("Failed to read APP_SECRET environment variable");
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

pub async fn authenticate(ctx: AppContext, user: &str, pw: &str) -> Result<String> {
    let secret =
        std::env::var("APP_SECRET").expect("Failed to read APP_SECRET environment variable");
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    match ctx.channel().hash_for(user) {
        Some(user_hash) => match password::verify(pw, user_hash) {
            Ok(false) => Err(Error::UserNotFound),
            Ok(true) => match token::make_token(
                WebSession {
                    user: user.to_string(),
                    expires: since_the_epoch
                        .checked_add(Duration::new(TOKEN_DURATION, 0))
                        .unwrap()
                        .as_secs(),
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
                error!("Password issue: {}", e);
                Err(Error::PasswordIssue)
            }
        },
        None => Err(Error::UserNotFound),
    }
}
