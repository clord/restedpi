use crate::i2c_bus::error::Error;
use crate::i2c_bus::i2c_io::{
    I2cAction::{Read, Write},
    I2cMessage,
};
use crate::i2c_bus::Result;

use std::vec::Vec;

use std::sync::mpsc::{channel, Sender};

pub struct Device {
    address: u16,
    state: unknown,
    i2c: Sender<I2cMessage>,
}

impl Device {
   configure() -> Result<Device>{
       let mut device = Device {
       
       }
       Ok(device)
   } 
}
