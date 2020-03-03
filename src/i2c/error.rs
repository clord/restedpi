use rppal::i2c;
use serde_derive::Serialize;
use std::error;
use std::fmt;
use std::io;
use std::sync::mpsc;

#[derive(Debug, Serialize)]
pub enum Error {
    IoError(String),
    InvalidPinDirection,
    NonExistant(String),
    OutOfBounds(usize),
    I2cError(String),
    RecvError(String),
    SendError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::IoError(ref err) => write!(f, "I/O error: {}", err),
            Error::InvalidPinDirection => write!(f, "Can't set pin to direction"),
            Error::NonExistant(ref name) => write!(f, "Device '{}' does not exist", name),
            Error::OutOfBounds(ref index) => {
                write!(f, "Device does not support index {:#?}", index)
            }
            Error::I2cError(ref err) => write!(f, "I2C Error: {}", err),
            Error::RecvError(ref err) => write!(f, "Recv Error: {}", err),
            Error::SendError(ref err) => write!(f, "Send Error: {}", err),
        }
    }
}

impl error::Error for Error {}

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
