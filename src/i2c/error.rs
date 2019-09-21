use crate::i2c::Unit;
use rppal::i2c;
use std::error;
use std::fmt;
use std::io;
use std::sync::mpsc;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidPinDirection,
    InvalidPinIndex,
    UnsupportedUnit(Unit),
    I2cError(i2c::Error),
    RecvError(mpsc::RecvError),
    SendError(mpsc::SendError<crate::i2c::bus::I2cMessage>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "I/O error: {}", err),
            Error::InvalidPinDirection => write!(f, "Can't set pin to direction"),
            Error::InvalidPinIndex => write!(f, "Pin index is invalid for device"),
            Error::UnsupportedUnit(ref unit) => {
                write!(f, "Device does not support unit {:#?}", unit)
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
        Error::Io(err)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Error {
        Error::RecvError(err)
    }
}
impl From<std::sync::mpsc::SendError<crate::i2c::bus::I2cMessage>> for Error {
    fn from(err: std::sync::mpsc::SendError<crate::i2c::bus::I2cMessage>) -> Error {
        Error::SendError(err)
    }
}

impl From<i2c::Error> for Error {
    fn from(err: i2c::Error) -> Error {
        Error::I2cError(err)
    }
}
