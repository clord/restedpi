use crate::i2c::Result;
use rppal::i2c::I2c;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use std::vec::Vec;

pub type Address = u16;
pub type Command = u8;

#[derive(Clone, Debug)]
pub enum I2cMessage {
    Write {
        address: Address,
        command: Command,
        parameters: Vec<u8>,
        response: Sender<Result<()>>,
    },
    Read {
        address: Address,
        command: Command,
        size: usize,
        response: Sender<Result<Vec<u8>>>,
    },
    Delay {
        duration: Duration,
        response: Sender<Result<()>>,
    },
}

/**
 * Represent the system I2C bus to arbitrary threads. read and write
 * actions are atomically performed, including any address change.
 * Delay command blocks the bus and prevents other actions.
 */
#[derive(Clone, Debug)]
pub struct I2cBus {
    sender: Sender<I2cMessage>,
}

impl I2cBus {
    pub fn write(&self, address: Address, command: u8, parameters: Vec<u8>) -> Result<()> {
        let (response, port) = channel();
        self.sender.send(I2cMessage::Write {
            parameters,
            response,
            address,
            command,
        })?;

        port.recv()?
    }

    pub fn read(&self, address: u16, command: u8, size: usize) -> Result<Vec<u8>> {
        let (response, port) = channel();
        self.sender.send(I2cMessage::Read {
            size,
            response,
            address,
            command,
        })?;

        port.recv()?
    }

    pub fn delay(&self, duration: Duration) -> Result<()> {
        let (response, port) = channel();
        self.sender.send(I2cMessage::Delay { duration, response })?;

        port.recv()?
    }
}

fn next_message(
    current_address: &mut Option<Address>,
    i2c: &mut I2c,
    receiver: &Receiver<I2cMessage>,
) {
    let next = receiver.recv().unwrap();
    match next {
        I2cMessage::Write {
            address,
            command,
            response,
            parameters,
        } => {
            if *current_address != Some(address) {
                i2c.set_slave_address(address);
                *current_address = Some(address);
            }
            let _result = i2c.block_write(command, &parameters);
            response.send(Ok(()));
        }

        I2cMessage::Delay { duration, response } => {
            thread::sleep(duration);
            response.send(Ok(()));
        }

        I2cMessage::Read {
            address,
            command,
            size,
            response,
        } => {
            if *current_address != Some(address) {
                i2c.set_slave_address(address);
                *current_address = Some(address);
            }
            debug!("read: {}, {}, {}", address, command, size);
            let mut vec = Vec::new();
            vec.resize(size, 0u8);
            match i2c.block_read(command, &mut vec) {
                Ok(()) => {
                    match response.send(Ok(vec)) {
                        Ok(()) => (), Err(e) => error!("Failed to send response: {:?}", e)
                    }
                }
                Err(e) => {
                    match response.send(Err(crate::i2c::error::Error::I2cError(e))) {
                        Ok(()) => (), Err(e) => error!("Failed to send response: {:?}", e)
                    }
                }
            };
        }
    };
}

pub fn start() -> I2cBus {
    let mut current_address: Option<Address> = None;
    let (sender, receiver) = channel::<I2cMessage>();

    thread::spawn(move || match I2c::new() {
        Ok(mut i2c) => loop {
            next_message(&mut current_address, &mut i2c, &receiver);
        },
        Err(UnknownModel) => {
            error!("Unsupported Raspberry PI; I2C bus not available");
        }
        Err(err) => {
            error!("There was a problem connecting to the I2C bus: {:?}", err);
            info!("The I2C bus connected to pins 3 and 5 is disabled by default");
            info!("Bus can be enabled with `sudo raspi-config`, or by adding `dtparam=i2c_arm=on` to `/boot/config.txt`");
            info!("(Remember to reboot the Raspberry Pi afterwards)");
        }
    });
    I2cBus { sender }
}
