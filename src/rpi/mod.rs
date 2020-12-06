use crate::error::Result;
use rppal::i2c::I2c;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::vec::Vec;

pub mod device;
pub mod i2c;

#[derive(Clone, Debug)]
pub enum RpiMessage {
    WriteI2C {
        address: i2c::I2cAddress,
        command: i2c::I2cCommand,
        parameters: Vec<u8>,
        response: Sender<Result<()>>,
    },
    ReadI2C {
        address: i2c::I2cAddress,
        command: i2c::I2cCommand,
        size: usize,
        response: Sender<Result<Vec<u8>>>,
    },
    ReadGpio {
        pin: u8,
        response: Sender<Result<rppal::gpio::Level>>,
    },
    // GpioRead (any gpio pin can have current level read)
    // GpioNotify (will write to a provided channel current value whenever the specified pin changes state)
    // GpioSetForWrite (required before writing to a pin (can still read))
    // GpioWrite (must be set to write, or will fail)
}

/**
 * Represent the system I2C bus to arbitrary threads. read and write
 * actions are atomically performed, including any address change.
 * Delay command blocks the bus and prevents other actions.
 */
#[derive(Clone, Debug)]
pub struct RpiApi {
    sender: Sender<RpiMessage>,
}

impl RpiApi {
    pub fn write_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        parameters: Vec<u8>,
    ) -> Result<()> {
        let (response, port) = channel();
        self.sender.send(RpiMessage::WriteI2C {
            parameters,
            response,
            address,
            command,
        })?;

        port.recv()?
    }

    pub fn read_i2c(&self, address: u16, command: u8, size: usize) -> Result<Vec<u8>> {
        let (response, port) = channel();
        self.sender.send(RpiMessage::ReadI2C {
            size,
            response,
            address,
            command,
        })?;

        port.recv()?
    }

    pub fn read_gpio(&self, pin: u8) -> Result<rppal::gpio::Level> {
        let (response, port) = channel();
        self.sender.send(RpiMessage::ReadGpio { response, pin })?;

        port.recv()?
    }
}

pub fn start() -> RpiApi {
    let (sender, receiver) = channel::<RpiMessage>();

    thread::spawn(move || {
        let mut current_i2c_address: Option<i2c::I2cAddress> = None;
        match I2c::new() {
            Ok(mut i2c) => loop {
                let next = receiver.recv().unwrap();
                match next {
                    RpiMessage::ReadGpio { pin, response } => {
                        debug!("TODO: read gpio {}", pin);
                        // TODO: Actually read the pin
                        response.send(Ok(rppal::gpio::Level::High));
                    }
                    RpiMessage::WriteI2C {
                        address,
                        command,
                        response,
                        parameters,
                    } => {
                        if current_i2c_address != Some(address) {
                            match i2c.set_slave_address(address) {
                                Ok(()) => current_i2c_address = Some(address),
                                Err(e) => error!("Failed to switch address: {}", e),
                            };
                        };
                        debug!("i2c write: {}, {}, {:?}", address, command, parameters);
                        let _result = i2c.block_write(command, &parameters);
                        match response.send(Ok(())) {
                            Ok(()) => (),
                            Err(e) => error!("Failed to respond in write: {}", e),
                        };
                    }

                    RpiMessage::ReadI2C {
                        address,
                        command,
                        size,
                        response,
                    } => {
                        if current_i2c_address != Some(address) {
                            match i2c.set_slave_address(address) {
                                Ok(()) => current_i2c_address = Some(address),
                                Err(e) => error!("Failed to switch address: {}", e),
                            };
                        }
                        let mut vec = Vec::new();
                        vec.resize(size, 0u8);
                        match i2c.block_read(command, &mut vec) {
                            Ok(()) => match response.send(Ok(vec)) {
                                Ok(()) => {
                                    debug!("i2c read result: {}, {}, {}", address, command, size)
                                }
                                Err(e) => error!("Failed to send response: {:?}", e),
                            },
                            Err(e) => {
                                match response
                                    .send(Err(crate::error::Error::I2cError(format!("{}", e))))
                                {
                                    Ok(()) => (),
                                    Err(e) => error!("Failed to send response: {:?}", e),
                                }
                            }
                        };
                    }
                };
            },
            Err(rppal::i2c::Error::UnknownModel) => {
                error!("Unsupported Raspberry PI; I2C bus not available");
            }
            Err(err) => {
                error!("There was a problem connecting to the I2C bus: {:?}", err);
                info!("The I2C bus connected to pins 3 and 5 is disabled by default");
                info!("Bus can be enabled with `sudo raspi-config`, or by adding `dtparam=i2c_arm=on` to `/boot/config.txt`");
                info!("(Remember to reboot the Raspberry Pi afterwards)");
            }
        }
    });
    RpiApi { sender }
}
