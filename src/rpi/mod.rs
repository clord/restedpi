use crate::error::Result;
#[cfg(feature = "raspberrypi")]
use rppal::i2c::I2c;
use std::vec::Vec;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{debug, error, info};

pub mod device;
pub mod i2c;

#[derive(Debug)]
pub enum RpiMessage {
    WriteI2C {
        address: i2c::I2cAddress,
        command: i2c::I2cCommand,
        parameters: Vec<u8>,
        response: oneshot::Sender<Result<()>>,
    },
    ReadI2C {
        address: i2c::I2cAddress,
        command: i2c::I2cCommand,
        size: usize,
        response: oneshot::Sender<Result<Vec<u8>>>,
    },
    #[cfg(feature = "raspberrypi")]
    ReadGpio {
        pin: u8,
        response: oneshot::Sender<Result<rppal::gpio::Level>>,
    },
    // GpioRead (any gpio pin can have current level read)
    // GpioNotify (will write to a provided channel current value whenever the specified pin changes state)
    // GpioSetForWrite (required before writing to a pin (can still read))
    // GpioWrite (must be set to write, or will fail)
}

/**
 * Represent the system I2C bus to arbitrary threads. read and write
 * actions are atomically performed, including any address change.
 */
#[derive(Clone, Debug)]
pub struct RpiApi {
    sender: mpsc::Sender<RpiMessage>,
}

impl RpiApi {
    pub async fn write_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        parameters: Vec<u8>,
    ) -> Result<()> {
        let (response, port) = oneshot::channel();

        self.sender
            .clone()
            .send(RpiMessage::WriteI2C {
                parameters,
                response,
                address,
                command,
            })
            .await?;

        port.await?
    }

    pub async fn read_i2c(&self, address: u16, command: u8, size: usize) -> Result<Vec<u8>> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::ReadI2C {
                size,
                response,
                address,
                command,
            })
            .await?;

        port.await?
    }

    #[cfg(feature = "raspberrypi")]
    pub async fn read_gpio(&self, pin: u8) -> Result<rppal::gpio::Level> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::ReadGpio { response, pin })
            .await?;

        port.await?
    }
}

pub fn start() -> RpiApi {
    #[cfg(feature = "raspberrypi")]
    let (sender, mut receiver) = mpsc::channel::<RpiMessage>(10);
    #[cfg(not(feature = "raspberrypi"))]
    let (sender, _receiver) = mpsc::channel::<RpiMessage>(10);

    #[cfg(feature = "raspberrypi")]
    tokio::spawn(async move {
        let mut current_i2c_address: Option<i2c::I2cAddress> = None;
        match I2c::new() {
            Ok(mut i2c) => loop {
                match receiver.recv().await {
                    Some(next) => match next {
                        RpiMessage::ReadGpio { pin, response } => {
                            debug!("TODO: read gpio {}", pin);
                            response.send(Ok(rppal::gpio::Level::High)).unwrap();
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
                                Err(e) => error!("Failed to respond in write: {:?}", e),
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
                                        debug!(
                                            "i2c read result: {}, {}, {}",
                                            address, command, size
                                        )
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
                    },
                    None => break,
                };
            },
            #[cfg(feature = "raspberrypi")]
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
