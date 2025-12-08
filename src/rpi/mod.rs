#[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
use crate::error::Error;
use crate::error::Result;
#[cfg(feature = "raspberrypi")]
use rppal::gpio::{Gpio, InputPin, OutputPin};
#[cfg(feature = "raspberrypi")]
use rppal::i2c::I2c;
#[cfg(any(feature = "raspberrypi", feature = "mock-gpio"))]
use std::collections::HashMap;
#[cfg(any(feature = "raspberrypi", feature = "mock-gpio"))]
use std::sync::Arc;
use std::vec::Vec;
#[cfg(any(feature = "raspberrypi", feature = "mock-gpio"))]
use tokio::sync::Mutex;

#[cfg(feature = "raspberrypi")]
use tracing::debug;

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
use tracing::debug;

pub mod device;
pub mod i2c;

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

// ============================================================================
// Real Raspberry Pi I2C and GPIO State
// ============================================================================

/// Tracks configured GPIO pins - either as input or output
#[cfg(feature = "raspberrypi")]
enum GpioPin {
    Input(InputPin),
    Output(OutputPin),
}

#[cfg(feature = "raspberrypi")]
struct RpiState {
    i2c: I2c,
    current_address: Option<i2c::I2cAddress>,
    gpio: Gpio,
    pins: HashMap<u8, GpioPin>,
}

#[cfg(feature = "raspberrypi")]
impl RpiState {
    fn new(bus: u8) -> Result<Self> {
        let i2c = I2c::with_bus(bus).map_err(|e| {
            crate::error::Error::I2cError(format!("Failed to open I2C bus {}: {}", bus, e))
        })?;
        let gpio = Gpio::new().map_err(|e| {
            crate::error::Error::DeviceReadError(format!("Failed to initialize GPIO: {}", e))
        })?;
        Ok(Self {
            i2c,
            current_address: None,
            gpio,
            pins: HashMap::new(),
        })
    }

    fn ensure_address(&mut self, address: i2c::I2cAddress) -> Result<()> {
        if self.current_address != Some(address) {
            self.i2c.set_slave_address(address).map_err(|e| {
                crate::error::Error::I2cError(format!(
                    "Failed to set I2C address {}: {}",
                    address, e
                ))
            })?;
            self.current_address = Some(address);
        }
        Ok(())
    }

    fn i2c_write(&mut self, address: i2c::I2cAddress, command: u8, data: &[u8]) -> Result<()> {
        self.ensure_address(address)?;
        debug!(
            "i2c write: addr={}, cmd={}, data={:?}",
            address, command, data
        );
        self.i2c
            .block_write(command, data)
            .map_err(|e| crate::error::Error::I2cError(format!("I2C write failed: {}", e)))?;
        Ok(())
    }

    fn i2c_read(&mut self, address: i2c::I2cAddress, command: u8, size: usize) -> Result<Vec<u8>> {
        self.ensure_address(address)?;
        let mut buffer = vec![0u8; size];
        self.i2c
            .block_read(command, &mut buffer)
            .map_err(|e| crate::error::Error::I2cError(format!("I2C read failed: {}", e)))?;
        debug!(
            "i2c read: addr={}, cmd={}, size={}, result={:?}",
            address, command, size, buffer
        );
        Ok(buffer)
    }

    fn gpio_read(&mut self, pin: u8) -> Result<rppal::gpio::Level> {
        // Check if we already have this pin configured
        if let Some(gpio_pin) = self.pins.get(&pin) {
            match gpio_pin {
                GpioPin::Input(input_pin) => {
                    let level = input_pin.read();
                    debug!("gpio read pin {}: {:?}", pin, level);
                    Ok(level)
                }
                GpioPin::Output(output_pin) => {
                    // Can still read from an output pin
                    let level = if output_pin.is_set_high() {
                        rppal::gpio::Level::High
                    } else {
                        rppal::gpio::Level::Low
                    };
                    debug!("gpio read (output) pin {}: {:?}", pin, level);
                    Ok(level)
                }
            }
        } else {
            // Configure as input and read
            let input_pin = self
                .gpio
                .get(pin)
                .map_err(|e| {
                    crate::error::Error::DeviceReadError(format!(
                        "Failed to get GPIO pin {}: {}",
                        pin, e
                    ))
                })?
                .into_input();
            let level = input_pin.read();
            debug!("gpio read pin {} (newly configured): {:?}", pin, level);
            self.pins.insert(pin, GpioPin::Input(input_pin));
            Ok(level)
        }
    }

    fn gpio_write(&mut self, pin: u8, level: rppal::gpio::Level) -> Result<()> {
        // Check if we already have this pin configured as output
        if let Some(gpio_pin) = self.pins.get_mut(&pin) {
            match gpio_pin {
                GpioPin::Output(output_pin) => {
                    match level {
                        rppal::gpio::Level::High => output_pin.set_high(),
                        rppal::gpio::Level::Low => output_pin.set_low(),
                    }
                    debug!("gpio write pin {}: {:?}", pin, level);
                    Ok(())
                }
                GpioPin::Input(_) => {
                    // Need to reconfigure as output
                    // Remove the old pin first
                    self.pins.remove(&pin);
                    let mut output_pin = self
                        .gpio
                        .get(pin)
                        .map_err(|e| {
                            crate::error::Error::DeviceReadError(format!(
                                "Failed to get GPIO pin {}: {}",
                                pin, e
                            ))
                        })?
                        .into_output();
                    match level {
                        rppal::gpio::Level::High => output_pin.set_high(),
                        rppal::gpio::Level::Low => output_pin.set_low(),
                    }
                    debug!(
                        "gpio write pin {} (reconfigured from input): {:?}",
                        pin, level
                    );
                    self.pins.insert(pin, GpioPin::Output(output_pin));
                    Ok(())
                }
            }
        } else {
            // Configure as output and write
            let mut output_pin = self
                .gpio
                .get(pin)
                .map_err(|e| {
                    crate::error::Error::DeviceReadError(format!(
                        "Failed to get GPIO pin {}: {}",
                        pin, e
                    ))
                })?
                .into_output();
            match level {
                rppal::gpio::Level::High => output_pin.set_high(),
                rppal::gpio::Level::Low => output_pin.set_low(),
            }
            debug!("gpio write pin {} (newly configured): {:?}", pin, level);
            self.pins.insert(pin, GpioPin::Output(output_pin));
            Ok(())
        }
    }
}

// ============================================================================
// Mock GPIO State (for testing without hardware)
// ============================================================================

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
#[derive(Debug, Default)]
struct MockState {
    pins: HashMap<u8, MockPinState>,
    /// Mock I2C device registers: address -> (register -> value)
    i2c_devices: HashMap<u16, HashMap<u8, Vec<u8>>>,
}

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
impl MockState {
    fn new() -> Self {
        Self::default()
    }

    fn read_pin(&self, pin: u8) -> GpioLevel {
        self.pins
            .get(&pin)
            .map(|s| s.level)
            .unwrap_or(GpioLevel::Low)
    }

    fn write_pin(&mut self, pin: u8, level: GpioLevel) {
        let state = self.pins.entry(pin).or_default();
        state.level = level;
    }

    fn set_pin_mode(&mut self, pin: u8, mode: MockPinMode) {
        let state = self.pins.entry(pin).or_default();
        state.mode = mode;
    }

    fn get_pin_state(&self, pin: u8) -> MockPinState {
        self.pins.get(&pin).cloned().unwrap_or_default()
    }

    fn i2c_write(&mut self, address: u16, register: u8, data: &[u8]) {
        let device = self.i2c_devices.entry(address).or_default();
        device.insert(register, data.to_vec());
    }

    fn i2c_read(&self, address: u16, register: u8, size: usize) -> Vec<u8> {
        if let Some(device) = self.i2c_devices.get(&address) {
            if let Some(data) = device.get(&register) {
                let mut result = data.clone();
                result.resize(size, 0);
                return result;
            }
        }
        // Return zeros for uninitialized registers
        vec![0u8; size]
    }

    fn set_i2c_register(&mut self, address: u16, register: u8, data: Vec<u8>) {
        let device = self.i2c_devices.entry(address).or_default();
        device.insert(register, data);
    }
}

// ============================================================================
// RpiApi - Direct async API (no message queue!)
// ============================================================================

/// Represent the system I2C bus and GPIO.
/// Uses a Mutex for thread-safe access - much simpler than message queues.
#[derive(Clone, Debug)]
pub struct RpiApi {
    #[cfg(feature = "raspberrypi")]
    state: Arc<Mutex<Option<RpiState>>>,

    #[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
    state: Arc<Mutex<MockState>>,

    #[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
    _phantom: std::marker::PhantomData<()>,
}

// Real Raspberry Pi implementation
#[cfg(feature = "raspberrypi")]
impl RpiApi {
    pub async fn write_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        parameters: Vec<u8>,
    ) -> Result<()> {
        let mut guard = self.state.lock().await;
        match guard.as_mut() {
            Some(rpi_state) => rpi_state.i2c_write(address, command, &parameters),
            None => Err(crate::error::Error::I2cError(
                "I2C bus not initialized".to_string(),
            )),
        }
    }

    pub async fn read_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        size: usize,
    ) -> Result<Vec<u8>> {
        let mut guard = self.state.lock().await;
        match guard.as_mut() {
            Some(rpi_state) => rpi_state.i2c_read(address, command, size),
            None => Err(crate::error::Error::I2cError(
                "I2C bus not initialized".to_string(),
            )),
        }
    }

    pub async fn read_gpio(&self, pin: u8) -> Result<rppal::gpio::Level> {
        let mut guard = self.state.lock().await;
        match guard.as_mut() {
            Some(rpi_state) => rpi_state.gpio_read(pin),
            None => Err(crate::error::Error::DeviceReadError(
                "GPIO not initialized".to_string(),
            )),
        }
    }

    pub async fn write_gpio(&self, pin: u8, level: rppal::gpio::Level) -> Result<()> {
        let mut guard = self.state.lock().await;
        match guard.as_mut() {
            Some(rpi_state) => rpi_state.gpio_write(pin, level),
            None => Err(crate::error::Error::DeviceReadError(
                "GPIO not initialized".to_string(),
            )),
        }
    }
}

// Mock implementation for testing
#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
impl RpiApi {
    pub async fn write_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        parameters: Vec<u8>,
    ) -> Result<()> {
        let mut guard = self.state.lock().await;
        debug!(
            "mock i2c write: addr={}, cmd={}, data={:?}",
            address, command, parameters
        );
        guard.i2c_write(address, command, &parameters);
        Ok(())
    }

    pub async fn read_i2c(
        &self,
        address: i2c::I2cAddress,
        command: u8,
        size: usize,
    ) -> Result<Vec<u8>> {
        let guard = self.state.lock().await;
        debug!(
            "mock i2c read: addr={}, cmd={}, size={}",
            address, command, size
        );
        Ok(guard.i2c_read(address, command, size))
    }

    pub async fn read_gpio(&self, pin: u8) -> Result<GpioLevel> {
        let guard = self.state.lock().await;
        let level = guard.read_pin(pin);
        debug!("mock read gpio {}: {:?}", pin, level);
        Ok(level)
    }

    pub async fn write_gpio(&self, pin: u8, level: GpioLevel) -> Result<()> {
        let mut guard = self.state.lock().await;
        debug!("mock write gpio {}: {:?}", pin, level);
        guard.write_pin(pin, level);
        Ok(())
    }

    pub async fn set_pin_mode(&self, pin: u8, mode: MockPinMode) -> Result<()> {
        let mut guard = self.state.lock().await;
        debug!("mock set pin mode {}: {:?}", pin, mode);
        guard.set_pin_mode(pin, mode);
        Ok(())
    }

    /// Get a copy of pin state for test assertions
    pub async fn get_pin_state(&self, pin: u8) -> MockPinState {
        let guard = self.state.lock().await;
        guard.get_pin_state(pin)
    }

    /// Pre-populate I2C register with test data (for test setup)
    pub async fn set_i2c_register(&self, address: u16, register: u8, data: Vec<u8>) {
        let mut guard = self.state.lock().await;
        guard.set_i2c_register(address, register, data);
    }
}

// Stub implementation when no GPIO feature is enabled
#[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
impl RpiApi {
    pub async fn write_i2c(
        &self,
        _address: i2c::I2cAddress,
        _command: u8,
        _parameters: Vec<u8>,
    ) -> Result<()> {
        Err(Error::DeviceReadError(
            "I2C unavailable: enable 'raspberrypi' or 'mock-gpio' feature".to_string(),
        ))
    }

    pub async fn read_i2c(
        &self,
        _address: i2c::I2cAddress,
        _command: u8,
        _size: usize,
    ) -> Result<Vec<u8>> {
        Err(Error::DeviceReadError(
            "I2C unavailable: enable 'raspberrypi' or 'mock-gpio' feature".to_string(),
        ))
    }

    pub async fn read_gpio(&self, _pin: u8) -> Result<GpioLevel> {
        Err(Error::DeviceReadError(
            "GPIO unavailable: enable 'raspberrypi' or 'mock-gpio' feature".to_string(),
        ))
    }
}

// ============================================================================
// Constructor
// ============================================================================

/// Create a new RpiApi instance.
/// For raspberrypi: initializes the I2C bus and GPIO
/// For mock-gpio: creates empty mock state
/// For neither: creates a stub that returns errors
#[cfg(feature = "raspberrypi")]
pub fn start(bus: u8) -> RpiApi {
    use tracing::{error, info};

    let rpi_state = match RpiState::new(bus) {
        Ok(state) => Some(state),
        Err(e) => {
            error!("Failed to initialize hardware: {}", e);
            info!("The I2C bus connected to pins 3 and 5 is disabled by default");
            info!(
                "Bus can be enabled with `sudo raspi-config`, or by adding `dtparam=i2c_arm=on` to `/boot/config.txt`"
            );
            info!("(Remember to reboot the Raspberry Pi afterwards)");
            None
        }
    };

    RpiApi {
        state: Arc::new(Mutex::new(rpi_state)),
    }
}

#[cfg(all(feature = "mock-gpio", not(feature = "raspberrypi")))]
pub fn start(_bus: u8) -> RpiApi {
    RpiApi {
        state: Arc::new(Mutex::new(MockState::new())),
    }
}

#[cfg(not(any(feature = "raspberrypi", feature = "mock-gpio")))]
pub fn start(_bus: u8) -> RpiApi {
    RpiApi {
        _phantom: std::marker::PhantomData,
    }
}
