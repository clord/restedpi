#[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
use crate::error::Error;
use crate::error::Result;
#[cfg(feature = "raspberrypi")]
use rppal::i2c::I2c;
use std::vec::Vec;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[cfg(feature = "raspberrypi")]
use tracing::{debug, error, info};

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
use std::collections::HashMap;
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
use std::sync::{Arc, Mutex};
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
use tracing::debug;

/// Type alias for mock I2C device registers: address -> (register -> value)
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
type MockI2cDevices = Arc<Mutex<HashMap<u16, HashMap<u8, Vec<u8>>>>>;

/// GPIO level for non-raspberrypi builds
#[cfg(not(feature = "raspberrypi"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioLevel {
    Low,
    High,
}

#[cfg(not(feature = "raspberrypi"))]
impl GpioLevel {
    /// Convert boolean to GPIO level (true = High, false = Low)
    #[must_use]
    pub fn from_bool(value: bool) -> Self {
        if value {
            GpioLevel::High
        } else {
            GpioLevel::Low
        }
    }

    /// Convert GPIO level to boolean (High = true, Low = false)
    #[must_use]
    pub fn to_bool(self) -> bool {
        match self {
            GpioLevel::High => true,
            GpioLevel::Low => false,
        }
    }
}

/// Mock GPIO pin configuration
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockPinMode {
    Input,
    Output,
}

/// State for a single mock GPIO pin
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
#[derive(Debug, Clone)]
pub struct MockPinState {
    pub mode: MockPinMode,
    pub level: GpioLevel,
    pub pull_up: bool,
    pub pull_down: bool,
}

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
impl Default for MockPinState {
    fn default() -> Self {
        Self {
            mode: MockPinMode::Input,
            level: GpioLevel::Low,
            pull_up: false,
            pull_down: false,
        }
    }
}

/// Mock GPIO state manager - tracks state of all GPIO pins
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
#[derive(Debug, Clone)]
pub struct MockGpioState {
    pins: Arc<Mutex<HashMap<u8, MockPinState>>>,
    /// Mock I2C device registers
    i2c_devices: MockI2cDevices,
}

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
impl MockGpioState {
    pub fn new() -> Self {
        Self {
            pins: Arc::new(Mutex::new(HashMap::new())),
            i2c_devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the current level of a pin
    pub fn read_pin(&self, pin: u8) -> GpioLevel {
        let pins = self.pins.lock().unwrap();
        pins.get(&pin).map(|s| s.level).unwrap_or(GpioLevel::Low)
    }

    /// Set the level of a pin (for output pins or simulating input)
    pub fn write_pin(&self, pin: u8, level: GpioLevel) {
        let mut pins = self.pins.lock().unwrap();
        let state = pins.entry(pin).or_default();
        state.level = level;
    }

    /// Set pin mode
    pub fn set_pin_mode(&self, pin: u8, mode: MockPinMode) {
        let mut pins = self.pins.lock().unwrap();
        let state = pins.entry(pin).or_default();
        state.mode = mode;
    }

    /// Get pin state
    pub fn get_pin_state(&self, pin: u8) -> MockPinState {
        let pins = self.pins.lock().unwrap();
        pins.get(&pin).cloned().unwrap_or_default()
    }

    /// Register a mock I2C device at an address
    pub fn register_i2c_device(&self, address: u16) {
        let mut devices = self.i2c_devices.lock().unwrap();
        devices.entry(address).or_default();
    }

    /// Write to a mock I2C device register
    pub fn i2c_write(&self, address: u16, register: u8, data: &[u8]) -> Result<()> {
        let mut devices = self.i2c_devices.lock().unwrap();
        let device = devices.entry(address).or_default();
        device.insert(register, data.to_vec());
        Ok(())
    }

    /// Read from a mock I2C device register
    pub fn i2c_read(&self, address: u16, register: u8, size: usize) -> Result<Vec<u8>> {
        let devices = self.i2c_devices.lock().unwrap();
        if let Some(device) = devices.get(&address) {
            if let Some(data) = device.get(&register) {
                let mut result = data.clone();
                result.resize(size, 0);
                return Ok(result);
            }
        }
        // Return zeros for uninitialized registers
        Ok(vec![0u8; size])
    }

    /// Pre-populate I2C register with test data
    pub fn set_i2c_register(&self, address: u16, register: u8, data: Vec<u8>) {
        let mut devices = self.i2c_devices.lock().unwrap();
        let device = devices.entry(address).or_default();
        device.insert(register, data);
    }
}

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
impl Default for MockGpioState {
    fn default() -> Self {
        Self::new()
    }
}

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
    #[cfg(feature = "raspberrypi")]
    WriteGpio {
        pin: u8,
        level: rppal::gpio::Level,
        response: oneshot::Sender<Result<()>>,
    },
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    ReadGpio {
        pin: u8,
        response: oneshot::Sender<Result<GpioLevel>>,
    },
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    WriteGpio {
        pin: u8,
        level: GpioLevel,
        response: oneshot::Sender<Result<()>>,
    },
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    SetPinMode {
        pin: u8,
        mode: MockPinMode,
        response: oneshot::Sender<Result<()>>,
    },
}

/// Represent the system I2C bus and GPIO to arbitrary threads.
/// Read and write actions are atomically performed, including any address change.
#[derive(Clone, Debug)]
pub struct RpiApi {
    sender: mpsc::Sender<RpiMessage>,
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    mock_state: MockGpioState,
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

    #[cfg(feature = "raspberrypi")]
    pub async fn write_gpio(&self, pin: u8, level: rppal::gpio::Level) -> Result<()> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::WriteGpio {
                response,
                pin,
                level,
            })
            .await?;

        port.await?
    }

    /// Mock GPIO read - reads from simulated pin state
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    pub async fn read_gpio(&self, pin: u8) -> Result<GpioLevel> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::ReadGpio { response, pin })
            .await?;

        port.await?
    }

    /// Mock GPIO write - writes to simulated pin state
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    pub async fn write_gpio(&self, pin: u8, level: GpioLevel) -> Result<()> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::WriteGpio {
                response,
                pin,
                level,
            })
            .await?;

        port.await?
    }

    /// Set GPIO pin mode (mock only)
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    pub async fn set_pin_mode(&self, pin: u8, mode: MockPinMode) -> Result<()> {
        let (response, port) = oneshot::channel();
        self.sender
            .clone()
            .send(RpiMessage::SetPinMode {
                response,
                pin,
                mode,
            })
            .await?;

        port.await?
    }

    /// Get access to mock state for test setup (mock only)
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    pub fn mock_state(&self) -> &MockGpioState {
        &self.mock_state
    }

    /// GPIO unavailable - returns error when neither raspberrypi nor mock-gpio feature is enabled
    #[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
    pub async fn read_gpio(&self, _pin: u8) -> Result<GpioLevel> {
        Err(Error::DeviceReadError(
            "GPIO unavailable: enable 'raspberrypi' or 'mock-gpio' feature".to_string(),
        ))
    }
}

pub fn start(#[cfg_attr(not(feature = "raspberrypi"), allow(unused))] bus: u8) -> RpiApi {
    #[cfg(feature = "raspberrypi")]
    let (sender, mut receiver) = mpsc::channel::<RpiMessage>(10);
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    let (sender, mut receiver) = mpsc::channel::<RpiMessage>(10);
    #[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
    let (sender, _receiver) = mpsc::channel::<RpiMessage>(10);

    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    let mock_state = MockGpioState::new();

    #[cfg(feature = "raspberrypi")]
    tokio::spawn(async move {
        let mut current_i2c_address: Option<i2c::I2cAddress> = None;
        match I2c::with_bus(bus) {
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

    // Mock GPIO handler loop - processes messages using simulated state
    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    {
        let mock_state_clone = mock_state.clone();
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Some(next) => match next {
                        RpiMessage::ReadGpio { pin, response } => {
                            let level = mock_state_clone.read_pin(pin);
                            debug!("mock read gpio {}: {:?}", pin, level);
                            let _ = response.send(Ok(level));
                        }
                        RpiMessage::WriteGpio {
                            pin,
                            level,
                            response,
                        } => {
                            debug!("mock write gpio {}: {:?}", pin, level);
                            mock_state_clone.write_pin(pin, level);
                            let _ = response.send(Ok(()));
                        }
                        RpiMessage::SetPinMode {
                            pin,
                            mode,
                            response,
                        } => {
                            debug!("mock set pin mode {}: {:?}", pin, mode);
                            mock_state_clone.set_pin_mode(pin, mode);
                            let _ = response.send(Ok(()));
                        }
                        RpiMessage::WriteI2C {
                            address,
                            command,
                            parameters,
                            response,
                        } => {
                            debug!(
                                "mock i2c write: addr={}, cmd={}, data={:?}",
                                address, command, parameters
                            );
                            let result = mock_state_clone.i2c_write(address, command, &parameters);
                            let _ = response.send(result);
                        }
                        RpiMessage::ReadI2C {
                            address,
                            command,
                            size,
                            response,
                        } => {
                            debug!(
                                "mock i2c read: addr={}, cmd={}, size={}",
                                address, command, size
                            );
                            let result = mock_state_clone.i2c_read(address, command, size);
                            let _ = response.send(result);
                        }
                    },
                    None => break,
                }
            }
        });
    }

    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    {
        RpiApi { sender, mock_state }
    }
    #[cfg(not(all(feature = "mock-gpio", not(feature = "raspberrypi"))))]
    {
        RpiApi { sender }
    }
}
