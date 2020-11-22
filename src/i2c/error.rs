use crate::config::value::Unit;
use rppal::i2c;
use serde_derive::Serialize;
use std::error;
use std::fmt;
use std::io;
use std::sync::mpsc;

#[derive(Debug, Serialize)]
pub enum Error {
    IoError(String),
    InputNotFound(String),
    OutputNotFound(String),
    InvalidPinDirection,
    NonExistant(String),
    OutOfBounds(usize),
    UnitError(Unit),
    I2cError(String),
    RecvError(String),
    SendError(String),
    StorageError(String),
    EncodingError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(ref err) => write!(f, "I/O error: {}", err),
            Error::InvalidPinDirection => write!(f, "Invalid pin direction"),
            Error::NonExistant(ref name) => write!(f, "'{}' does not exist", name),
            Error::OutOfBounds(ref index) => write!(f, "Index '{:#?}' out of bounds", index),
            Error::I2cError(ref err) => write!(f, "I2C Bus Error: {}", err),
            Error::UnitError(ref err) => write!(f, "Unit expected {:#?}", err),
            Error::RecvError(ref err) => write!(f, "Failed to read: {}", err),
            Error::SendError(ref err) => write!(f, "Failed to send: {}", err),
            Error::StorageError(ref err) => write!(f, "Storage error: {}", err),
            Error::EncodingError(ref err) => write!(f, "Encoding error: {}", err),
            Error::InputNotFound(n) => write!(f, "Input error: {}", n),
            Error::OutputNotFound(n) => write!(f, "Output error: {}", n),
        }
    }
}

impl error::Error for Error {}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Error {
        Error::EncodingError(format!("ser: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::EncodingError(format!("de: {}", err))
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::EncodingError(format!("{}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(format!("{}", err))
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Error {
        Error::RecvError(format!("{}", err))
    }
}
impl From<std::sync::mpsc::SendError<crate::app::channel::AppMessage>> for Error {
    fn from(err: std::sync::mpsc::SendError<crate::app::channel::AppMessage>) -> Error {
        Error::SendError(format!("{}", err))
    }
}
impl From<std::sync::mpsc::SendError<crate::i2c::bus::I2cMessage>> for Error {
    fn from(err: std::sync::mpsc::SendError<crate::i2c::bus::I2cMessage>) -> Error {
        Error::SendError(format!("{}", err))
    }
}

impl From<i2c::Error> for Error {
    fn from(err: i2c::Error) -> Error {
        Error::I2cError(format!("{}", err))
    }
}
