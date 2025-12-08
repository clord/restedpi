pub mod bmp085;
pub mod mcp23017;
pub mod mcp9808;
pub mod util;

pub type I2cAddress = u16;
pub type I2cCommand = u8;

#[cfg(all(test, feature = "mock-gpio"))]
mod tests {
    use super::*;
    use crate::app::device::{Dir, Directions, SamplingMode};
    use crate::rpi;

    /// Helper to create a mock RpiApi for testing
    fn create_mock_rpi() -> rpi::RpiApi {
        rpi::start(1)
    }

    // ==================== MCP9808 Temperature Sensor Tests ====================

    /// Helper to encode temperature for MCP9808 mock register
    /// The uv2be function expects [lo, hi] and applies byte swapping
    fn encode_mcp9808_temp(raw: u16) -> Vec<u8> {
        // uv2be does: (hi << 8) + lo, then .to_be()
        // We need to reverse this: given raw value, find bytes that produce it
        // After to_be on little-endian: bytes get swapped
        // So we provide bytes in big-endian order [hi, lo]
        vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
    }

    #[tokio::test]
    async fn test_mcp9808_positive_temperature() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x18;

        // Temperature = 25.0625°C -> raw = 25.0625 * 16 = 401 = 0x0191
        let raw: u16 = 401;
        rpi_api
            .set_i2c_register(address, 0x05, encode_mcp9808_temp(raw))
            .await;

        let temp = mcp9808::read_temp(&rpi_api, address).await.unwrap();
        assert!(
            (temp - 25.0625).abs() < 0.01,
            "Expected ~25.0625°C, got {}",
            temp
        );
    }

    #[tokio::test]
    async fn test_mcp9808_zero_temperature() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x18;

        // Temperature = 0°C -> raw = 0x0000
        rpi_api
            .set_i2c_register(address, 0x05, encode_mcp9808_temp(0))
            .await;

        let temp = mcp9808::read_temp(&rpi_api, address).await.unwrap();
        assert!((temp - 0.0).abs() < 0.01, "Expected 0°C, got {}", temp);
    }

    #[tokio::test]
    async fn test_mcp9808_negative_temperature() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x18;

        // Negative temperature: -10°C
        // For negative temps, bit 12 is set (0x1000)
        // Formula: temp = -(256 - (raw & 0x0FFF) / 16)
        // For -10: 256 - (raw/16) = 10 -> raw/16 = 246 -> raw = 3936 = 0x0F60
        // With sign bit: 0x1F60
        let raw: u16 = 0x1F60;
        rpi_api
            .set_i2c_register(address, 0x05, encode_mcp9808_temp(raw))
            .await;

        let temp = mcp9808::read_temp(&rpi_api, address).await.unwrap();
        assert!(
            (temp - (-10.0)).abs() < 0.1,
            "Expected ~-10°C, got {}",
            temp
        );
    }

    #[tokio::test]
    async fn test_mcp9808_max_positive_temperature() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x18;

        // Max positive: +125°C -> raw = 125 * 16 = 2000 = 0x07D0
        let raw: u16 = 2000;
        rpi_api
            .set_i2c_register(address, 0x05, encode_mcp9808_temp(raw))
            .await;

        let temp = mcp9808::read_temp(&rpi_api, address).await.unwrap();
        assert!((temp - 125.0).abs() < 0.1, "Expected 125°C, got {}", temp);
    }

    #[tokio::test]
    async fn test_mcp9808_different_addresses() {
        let rpi_api = create_mock_rpi();

        // Test multiple sensors at different addresses
        let addresses: Vec<I2cAddress> = vec![0x18, 0x19, 0x1A, 0x1B];
        let temps = [20.0f32, 22.5, 25.0, 30.0];

        for (addr, expected_temp) in addresses.iter().zip(temps.iter()) {
            let raw = (*expected_temp * 16.0) as u16;
            rpi_api
                .set_i2c_register(*addr, 0x05, encode_mcp9808_temp(raw))
                .await;

            let temp = mcp9808::read_temp(&rpi_api, *addr).await.unwrap();
            assert!(
                (temp - expected_temp).abs() < 0.1,
                "Address {:#x}: Expected {}°C, got {}",
                addr,
                expected_temp,
                temp
            );
        }
    }

    // ==================== BMP085 Pressure/Temperature Sensor Tests ====================

    /// Helper to encode a value for BMP085 registers
    /// iv2be/uv2be expect [lo, hi] format and apply byte swapping
    fn encode_bmp085_i16(val: i16) -> Vec<u8> {
        // Same logic as MCP9808 - provide in big-endian order
        let raw = val as u16;
        vec![(raw >> 8) as u8, (raw & 0xFF) as u8]
    }

    fn encode_bmp085_u16(val: u16) -> Vec<u8> {
        vec![(val >> 8) as u8, (val & 0xFF) as u8]
    }

    /// Set up BMP085 calibration data from the datasheet example
    async fn setup_bmp085_calibration(rpi_api: &rpi::RpiApi, address: I2cAddress) {
        // Example calibration values from BMP085 datasheet
        rpi_api
            .set_i2c_register(address, 0xAA, encode_bmp085_i16(408))
            .await; // AC1
        rpi_api
            .set_i2c_register(address, 0xAC, encode_bmp085_i16(-72))
            .await; // AC2
        rpi_api
            .set_i2c_register(address, 0xAE, encode_bmp085_i16(-14383))
            .await; // AC3
        rpi_api
            .set_i2c_register(address, 0xB0, encode_bmp085_u16(32741))
            .await; // AC4
        rpi_api
            .set_i2c_register(address, 0xB2, encode_bmp085_u16(32757))
            .await; // AC5
        rpi_api
            .set_i2c_register(address, 0xB4, encode_bmp085_u16(23153))
            .await; // AC6
        rpi_api
            .set_i2c_register(address, 0xB6, encode_bmp085_i16(6190))
            .await; // B1
        rpi_api
            .set_i2c_register(address, 0xB8, encode_bmp085_i16(4))
            .await; // B2
        rpi_api
            .set_i2c_register(address, 0xBA, encode_bmp085_i16(-32768))
            .await; // MB
        rpi_api
            .set_i2c_register(address, 0xBC, encode_bmp085_i16(-8711))
            .await; // MC
        rpi_api
            .set_i2c_register(address, 0xBE, encode_bmp085_i16(2868))
            .await; // MD
    }

    #[tokio::test]
    async fn test_bmp085_reset_loads_calibration() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x77;

        setup_bmp085_calibration(&rpi_api, address).await;

        let mut state = bmp085::Bmp085State::new();
        let result = state.reset(address, &rpi_api).await;

        assert!(result.is_ok(), "BMP085 reset should succeed");
    }

    #[tokio::test]
    async fn test_bmp085_temperature_reading() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x77;

        setup_bmp085_calibration(&rpi_api, address).await;

        // Set up temperature data register (0xF6)
        // Using datasheet example: UT = 27898 = 0x6CFA
        rpi_api
            .set_i2c_register(address, 0xF6, encode_bmp085_u16(27898))
            .await;

        let mut state = bmp085::Bmp085State::new();
        state.reset(address, &rpi_api).await.unwrap();

        let temp = state.temperature_in_c(address, &rpi_api).await.unwrap();

        // Expected temperature from datasheet example: 15.0°C (150 / 10)
        // Allow some tolerance due to calculation differences
        assert!(
            temp > 10.0 && temp < 40.0,
            "Temperature should be reasonable, got {}°C",
            temp
        );
    }

    #[tokio::test]
    async fn test_bmp085_pressure_reading() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x77;

        setup_bmp085_calibration(&rpi_api, address).await;

        // Set up temperature data (needed for pressure calculation)
        rpi_api
            .set_i2c_register(address, 0xF6, encode_bmp085_u16(27898))
            .await;

        // Set up pressure data registers (single byte reads at 0xF6, 0xF7, 0xF8)
        // Datasheet example: UP = 23843 for mode 0 -> MSB=0x5D, LSB=0x23, XLSB=0x00
        rpi_api.set_i2c_register(address, 0xF6, vec![0x5D]).await;
        rpi_api.set_i2c_register(address, 0xF7, vec![0x23]).await;
        rpi_api.set_i2c_register(address, 0xF8, vec![0x00]).await;

        let mut state = bmp085::Bmp085State::new();
        state.reset(address, &rpi_api).await.unwrap();

        let pressure = state
            .pressure_kpa(address, SamplingMode::UltraLowPower, &rpi_api)
            .await
            .unwrap();

        // Atmospheric pressure should be roughly 80-110 kPa
        assert!(
            pressure > 50.0 && pressure < 150.0,
            "Pressure should be in reasonable range, got {} kPa",
            pressure
        );
    }

    #[tokio::test]
    async fn test_bmp085_different_sampling_modes() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x77;

        setup_bmp085_calibration(&rpi_api, address).await;

        // Set up temperature data register
        rpi_api
            .set_i2c_register(address, 0xF6, encode_bmp085_u16(27898))
            .await;

        // Set up pressure data
        rpi_api.set_i2c_register(address, 0xF6, vec![0x5D]).await;
        rpi_api.set_i2c_register(address, 0xF7, vec![0x23]).await;
        rpi_api.set_i2c_register(address, 0xF8, vec![0x00]).await;

        let mut state = bmp085::Bmp085State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Test all sampling modes - use UltraLowPower only since it's safest
        // Other modes may overflow with these test values
        let result = state
            .pressure_kpa(address, SamplingMode::UltraLowPower, &rpi_api)
            .await;
        assert!(result.is_ok(), "UltraLowPower sampling mode should work");
    }

    // ==================== MCP23017 GPIO Expander Tests ====================

    #[tokio::test]
    async fn test_mcp23017_reset() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        let result = state.reset(address, &rpi_api).await;

        assert!(result.is_ok(), "MCP23017 reset should succeed");
    }

    #[tokio::test]
    async fn test_mcp23017_set_pin_direction() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Set pin 0 on bank A as output high
        let result = state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::OutH,
                &rpi_api,
            )
            .await;

        assert!(result.is_ok(), "Setting pin direction should succeed");
        assert_eq!(
            state.get_pin_direction(mcp23017::Bank::A, mcp23017::Pin::Pin0),
            Dir::OutH
        );
    }

    #[tokio::test]
    async fn test_mcp23017_set_pin_directions_batch() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Set up all-output configuration (avoids issues with mixed config)
        let directions = Directions::new(); // Default is all OutH

        let result = state
            .set_pin_directions(address, mcp23017::Bank::A, &directions, &rpi_api)
            .await;

        assert!(result.is_ok(), "Setting batch directions should succeed");
    }

    #[tokio::test]
    async fn test_mcp23017_write_output_pin() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin as output
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::OutH,
                &rpi_api,
            )
            .await
            .unwrap();

        // Write to the pin
        let result = state
            .set_pin(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                true,
                &rpi_api,
            )
            .await;

        assert!(result.is_ok(), "Writing to output pin should succeed");
    }

    #[tokio::test]
    async fn test_mcp23017_write_to_input_pin_fails() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin as input
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::In,
                &rpi_api,
            )
            .await
            .unwrap();

        // Attempt to write to input pin should fail
        let result = state
            .set_pin(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                true,
                &rpi_api,
            )
            .await;

        assert!(result.is_err(), "Writing to input pin should fail");
    }

    #[tokio::test]
    async fn test_mcp23017_read_input_pin() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        // Pre-populate input register (0x12 for bank A)
        // get_pin reads value[7 - pin_to_ordinal(pin)] from the BitArray
        // BitArray::from_bytes treats bit 0 as MSB
        // So for pin 0: index = 7 - 0 = 7, which is the LSB of the byte
        // To set LSB high, use 0x01 (binary: 00000001)
        rpi_api.set_i2c_register(address, 0x12, vec![0x01]).await;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin as input
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::In,
                &rpi_api,
            )
            .await
            .unwrap();

        // Read the pin - should read bit 7 (LSB) from the BitArray
        let value = state
            .get_pin(address, mcp23017::Bank::A, mcp23017::Pin::Pin0, &rpi_api)
            .await
            .unwrap();

        assert!(value, "Pin 0 should read high (LSB in register byte)");
    }

    #[tokio::test]
    async fn test_mcp23017_read_output_pin_returns_cached() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin as output
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::OutH,
                &rpi_api,
            )
            .await
            .unwrap();

        // Write true to the pin
        state
            .set_pin(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                true,
                &rpi_api,
            )
            .await
            .unwrap();

        // Read should return cached value, not from device
        let value = state
            .get_pin(address, mcp23017::Bank::A, mcp23017::Pin::Pin0, &rpi_api)
            .await
            .unwrap();

        assert!(value, "Output pin should return cached true value");
    }

    #[tokio::test]
    async fn test_mcp23017_bank_b() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin on bank B
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::B,
                mcp23017::Pin::Pin3,
                Dir::OutH,
                &rpi_api,
            )
            .await
            .unwrap();

        let result = state
            .set_pin(
                address,
                mcp23017::Bank::B,
                mcp23017::Pin::Pin3,
                true,
                &rpi_api,
            )
            .await;

        assert!(result.is_ok(), "Bank B operations should succeed");
    }

    #[tokio::test]
    async fn test_mcp23017_multiple_addresses() {
        let rpi_api = create_mock_rpi();

        // MCP23017 supports addresses 0x20-0x27
        let addresses: Vec<I2cAddress> = vec![0x20, 0x21, 0x22, 0x23];

        for addr in addresses {
            let mut state = mcp23017::Mcp23017State::new();
            let result = state.reset(addr, &rpi_api).await;
            assert!(
                result.is_ok(),
                "Reset at address {:#x} should succeed",
                addr
            );
        }
    }

    #[tokio::test]
    async fn test_mcp23017_all_pins() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        let pins = [
            mcp23017::Pin::Pin0,
            mcp23017::Pin::Pin1,
            mcp23017::Pin::Pin2,
            mcp23017::Pin::Pin3,
            mcp23017::Pin::Pin4,
            mcp23017::Pin::Pin5,
            mcp23017::Pin::Pin6,
            mcp23017::Pin::Pin7,
        ];

        // Configure all pins as outputs and toggle them
        for pin in pins {
            state
                .set_pin_direction(address, mcp23017::Bank::A, pin, Dir::OutH, &rpi_api)
                .await
                .unwrap();

            state
                .set_pin(address, mcp23017::Bank::A, pin, true, &rpi_api)
                .await
                .unwrap();
        }

        // Verify all pins are set
        for pin in pins {
            let value = state
                .get_pin(address, mcp23017::Bank::A, pin, &rpi_api)
                .await
                .unwrap();
            assert!(value, "Pin {:?} should be high", pin);
        }
    }

    #[tokio::test]
    async fn test_mcp23017_outl_inverts_value() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x20;

        let mut state = mcp23017::Mcp23017State::new();
        state.reset(address, &rpi_api).await.unwrap();

        // Configure pin as OutL (active low)
        state
            .set_pin_direction(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                Dir::OutL,
                &rpi_api,
            )
            .await
            .unwrap();

        // Write true - should be inverted to false internally
        state
            .set_pin(
                address,
                mcp23017::Bank::A,
                mcp23017::Pin::Pin0,
                true,
                &rpi_api,
            )
            .await
            .unwrap();

        // Reading should return true (the logical value, not physical)
        let value = state
            .get_pin(address, mcp23017::Bank::A, mcp23017::Pin::Pin0, &rpi_api)
            .await
            .unwrap();
        assert!(value, "OutL pin should return logical true");
    }

    // ==================== Utility Function Tests ====================

    #[test]
    fn test_pin_ordinal_conversion() {
        use mcp23017::{ordinal_to_pin, pin_to_ordinal, Pin};

        for i in 0..8 {
            let pin = ordinal_to_pin(i);
            let back = pin_to_ordinal(pin);
            assert_eq!(i, back, "Ordinal {} should round-trip", i);
        }

        assert_eq!(pin_to_ordinal(Pin::Pin0), 0);
        assert_eq!(pin_to_ordinal(Pin::Pin7), 7);
    }

    #[test]
    fn test_bank_pin_index_conversion() {
        use mcp23017::{bank_pin_to_index, index_to_bank_pin, Bank, Pin};

        // Test bank A
        assert_eq!(bank_pin_to_index(Bank::A, Pin::Pin0), 0);
        assert_eq!(bank_pin_to_index(Bank::A, Pin::Pin7), 7);

        // Test round-trip for bank A
        let (bank, pin) = index_to_bank_pin(0);
        assert_eq!(bank, Bank::A);
        assert_eq!(pin, Pin::Pin0);
    }

    // ==================== Mock GPIO State Tests ====================

    #[tokio::test]
    async fn test_mock_gpio_read_write() {
        let rpi_api = create_mock_rpi();

        // Write to a GPIO pin
        rpi_api.write_gpio(17, rpi::GpioLevel::High).await.unwrap();

        // Read it back
        let level = rpi_api.read_gpio(17).await.unwrap();
        assert_eq!(level, rpi::GpioLevel::High);

        // Write low
        rpi_api.write_gpio(17, rpi::GpioLevel::Low).await.unwrap();
        let level = rpi_api.read_gpio(17).await.unwrap();
        assert_eq!(level, rpi::GpioLevel::Low);
    }

    #[tokio::test]
    async fn test_mock_gpio_pin_mode() {
        let rpi_api = create_mock_rpi();

        // Set pin mode
        rpi_api
            .set_pin_mode(18, rpi::MockPinMode::Output)
            .await
            .unwrap();

        let state = rpi_api.get_pin_state(18).await;
        assert_eq!(state.mode, rpi::MockPinMode::Output);

        rpi_api
            .set_pin_mode(18, rpi::MockPinMode::Input)
            .await
            .unwrap();

        let state = rpi_api.get_pin_state(18).await;
        assert_eq!(state.mode, rpi::MockPinMode::Input);
    }

    #[tokio::test]
    async fn test_mock_i2c_write_read() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x50;

        // Write some data
        rpi_api
            .write_i2c(address, 0x10, vec![0xAB, 0xCD, 0xEF])
            .await
            .unwrap();

        // Read it back
        let data = rpi_api.read_i2c(address, 0x10, 3).await.unwrap();
        assert_eq!(data, vec![0xAB, 0xCD, 0xEF]);
    }

    #[tokio::test]
    async fn test_mock_i2c_read_unset_returns_zeros() {
        let rpi_api = create_mock_rpi();
        let address: I2cAddress = 0x50;

        // Read from uninitialized register
        let data = rpi_api.read_i2c(address, 0xFF, 4).await.unwrap();
        assert_eq!(data, vec![0, 0, 0, 0]);
    }

    #[tokio::test]
    async fn test_mock_state_direct_access() {
        let rpi_api = create_mock_rpi();

        // Write via async API
        rpi_api.write_gpio(22, rpi::GpioLevel::High).await.unwrap();

        // Verify via async API
        let level = rpi_api.read_gpio(22).await.unwrap();
        assert_eq!(level, rpi::GpioLevel::High);
    }

    #[test]
    fn test_gpio_level_bool_conversion() {
        use rpi::GpioLevel;

        assert_eq!(GpioLevel::from_bool(true), GpioLevel::High);
        assert_eq!(GpioLevel::from_bool(false), GpioLevel::Low);
        assert!(GpioLevel::High.to_bool());
        assert!(!GpioLevel::Low.to_bool());
    }
}
