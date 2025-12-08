use hex::FromHexError;
use juniper::graphql_value;
use juniper::{FieldError, IntoFieldError};
#[cfg(feature = "raspberrypi")]
use rppal::i2c;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::io;
use std::sync::mpsc;

/// Represent all common results of i2c
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Serialize)]
#[must_use]
pub enum Error {
    Config(String),
    IoError(String),
    InputNotFound(String),
    OutputNotFound(String),
    DbError(String),
    InvalidPinDirection,
    ParseError,
    UserNotFound,
    TokenIssue,
    PasswordIssue,
    NotLoggedIn,
    DeviceReadError(String),
    PbkError(String),
    NonExistant(String),
    NotUnique(String),
    OutOfBounds(usize),
    UnitError(String),
    TzError(String),
    #[cfg(feature = "raspberrypi")]
    I2cError(String),
    RecvError(String),
    SendError(String),
    StorageError(String),
    EncodingError(String),
}

impl IntoFieldError for Error {
    fn into_field_error(self) -> FieldError {
        match self {
            Error::PbkError(ref err) => FieldError::new(err, graphql_value!({"slug": "PBK"})),
            Error::Config(ref err) => FieldError::new(err, graphql_value!({"slug": "Config"})),
            Error::IoError(ref err) => FieldError::new(err, graphql_value!({"slug": "IO"})),
            Error::DbError(ref err) => FieldError::new(err, graphql_value!({"slug": "DB"})),
            Error::DeviceReadError(ref err) => {
                FieldError::new(err, graphql_value!({"slug": "device-read"}))
            }
            Error::TzError(ref err) => FieldError::new(err, graphql_value!({"slug": "TZ"})),
            Error::NonExistant(ref name) => {
                FieldError::new(name, graphql_value!({"slug": "Existance"}))
            }
            Error::NotUnique(ref msg) => {
                FieldError::new(msg, graphql_value!({"slug": "Uniqueness"}))
            }
            Error::OutOfBounds(ref index) => {
                FieldError::new(index, graphql_value!({"slug": "Bounds"}))
            }
            #[cfg(feature = "raspberrypi")]
            Error::I2cError(ref err) => FieldError::new(err, graphql_value!({"slug": "I2C"})),
            Error::UnitError(ref err) => FieldError::new(err, graphql_value!({"slug": "Units"})),
            Error::RecvError(ref err) => FieldError::new(err, graphql_value!({"slug": "Recv"})),
            Error::SendError(ref err) => FieldError::new(err, graphql_value!({"slug": "Send"})),
            Error::StorageError(ref err) => {
                FieldError::new(err, graphql_value!({"slug": "Storage"}))
            }
            Error::EncodingError(ref err) => {
                FieldError::new(err, graphql_value!({"slug": "Encoding"}))
            }
            Error::InputNotFound(n) => FieldError::new(n, graphql_value!({"slug": "Input"})),
            Error::OutputNotFound(n) => FieldError::new(n, graphql_value!({"slug": "Output"})),
            Error::InvalidPinDirection => FieldError::new(
                "Direction incorrect",
                graphql_value!({"slug": "InvalidPinDirection"}),
            ),
            Error::ParseError => {
                FieldError::new("Failed to parse", graphql_value!({"slug": "Parser"}))
            }
            Error::UserNotFound => FieldError::new(String::new(), graphql_value!({"slug": "User"})),
            Error::TokenIssue => FieldError::new(
                "Failed to work with token",
                graphql_value!({"slug": "Token"}),
            ),
            Error::PasswordIssue => {
                FieldError::new("Password issue", graphql_value!({"slug": "Password"}))
            }
            Error::NotLoggedIn => {
                FieldError::new("Not Logged In", graphql_value!({"slug": "Login"}))
            }
        }
    }
}

impl warp::reject::Reject for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PbkError(ref err) => write!(f, "PBK error: {}", err),
            Error::Config(ref err) => write!(f, "Configuration error: {}", err),
            Error::IoError(ref err) => write!(f, "I/O error: {}", err),
            Error::DbError(ref err) => write!(f, "DB error: {}", err),
            Error::TzError(ref err) => write!(f, "TZ error: {}", err),
            Error::InvalidPinDirection => write!(f, "Invalid pin direction"),
            Error::ParseError => write!(f, "Parse error"),
            Error::DeviceReadError(ref err) => write!(f, "Failed to read device: {}", err),
            Error::NonExistant(ref name) => write!(f, "'{}' does not exist", name),
            Error::NotUnique(ref msg) => write!(f, "non-unique: {}", msg),
            Error::OutOfBounds(ref index) => write!(f, "Index '{:#?}' out of bounds", index),
            #[cfg(feature = "raspberrypi")]
            Error::I2cError(ref err) => write!(f, "I2C Bus Error: {}", err),
            Error::UnitError(ref err) => write!(f, "Unit expected {:#?}", err),
            Error::RecvError(ref err) => write!(f, "Failed to read: {}", err),
            Error::SendError(ref err) => write!(f, "Failed to send: {}", err),
            Error::StorageError(ref err) => write!(f, "Storage error: {}", err),
            Error::EncodingError(ref err) => write!(f, "Encoding error: {}", err),
            Error::InputNotFound(n) => write!(f, "Input not found: {}", n),
            Error::OutputNotFound(n) => write!(f, "Output not found: {}", n),
            Error::UserNotFound => write!(f, "User not found"),
            Error::TokenIssue => write!(f, "Issue with token"),
            Error::PasswordIssue => write!(f, "Password issue"),
            Error::NotLoggedIn => write!(f, "Not Logged In"),
        }
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(err: diesel::ConnectionError) -> Error {
        Error::DbError(format!("DB error: {}", err))
    }
}

impl error::Error for Error {}
impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error {
        Error::EncodingError(format!("ser: {}", err))
    }
}

impl From<FromHexError> for Error {
    fn from(err: FromHexError) -> Error {
        Error::EncodingError(format!("hex encoding error: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::EncodingError(format!("toml encoding error: {}", err))
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::EncodingError(format!("json encoding error: {}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(format!("io error: {}", err))
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Error {
        Error::RecvError(format!("mpsc recv: {}", err))
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(err: tokio::sync::oneshot::error::RecvError) -> Error {
        Error::RecvError(format!("tokio recv error: {}", err))
    }
}

impl From<std::sync::mpsc::SendError<crate::app::channel::AppMessage>> for Error {
    fn from(err: std::sync::mpsc::SendError<crate::app::channel::AppMessage>) -> Error {
        Error::SendError(format!("mpsc send error: {}", err))
    }
}

impl From<std::sync::mpsc::SendError<HashMap<std::string::String, crate::app::output::Output>>>
    for Error
{
    fn from(
        err: std::sync::mpsc::SendError<HashMap<std::string::String, crate::app::output::Output>>,
    ) -> Error {
        Error::SendError(format!("{}", err))
    }
}
impl From<std::sync::mpsc::SendError<HashMap<std::string::String, crate::app::input::Input>>>
    for Error
{
    fn from(
        err: std::sync::mpsc::SendError<HashMap<std::string::String, crate::app::input::Input>>,
    ) -> Error {
        Error::SendError(format!("{}", err))
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::app::channel::AppMessage>> for Error {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::app::channel::AppMessage>) -> Error {
        Error::SendError(format!("tokio send error: {}", err))
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        match err {
            diesel::result::Error::NotFound => {
                Error::NonExistant("Database Entry Not Found".to_string())
            }
            diesel::result::Error::DatabaseError(x, e) => match x {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    Error::NotUnique(format!("{:?}", e))
                }
                _ => Error::DbError(format!("Database Error {:?}: {:?}", x, e)),
            },
            _ => Error::DbError(format!("db error: {}", err)),
        }
    }
}

#[cfg(feature = "raspberrypi")]
impl From<i2c::Error> for Error {
    fn from(err: i2c::Error) -> Error {
        Error::I2cError(format!("i2c error: {}", err))
    }
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        Error::DbError(format!("r2d2: {}", err))
    }
}
