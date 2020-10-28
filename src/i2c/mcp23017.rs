use crate::i2c::bus::{Address, I2cBus};
use crate::i2c::{error::Error, Result};

const REGISTER_GPIOA: u8 = 0x00;
const REGISTER_GPIOB: u8 = 0x01;
const REGISTER_GPIOA_PULLUP: u8 = 0x0C;
const REGISTER_GPIOB_PULLUP: u8 = 0x0D;
const READ_GPIOA_ADDR: u8 = 0x12;
const READ_GPIOB_ADDR: u8 = 0x13;
const WRITE_GPIOA_ADDR: u8 = 0x14;
const WRITE_GPIOB_ADDR: u8 = 0x15;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Pullup {
    On,
    Off,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Direction {
    Output,
    Input(Pullup),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Pin {
    Pin0,
    Pin1,
    Pin2,
    Pin3,
    Pin4,
    Pin5,
    Pin6,
    Pin7,
}

fn direction_as_pullup_word(ps: [Direction; 8]) -> u8 {
    let mut result = 0u8;
    let mut dex = 0;
    for x in ps.iter() {
        if let Direction::Input(Pullup::On) = x {
            result = result | 1u8 >> dex;
        }
        dex += 1;
    }
    result
}

fn direction_as_inout_word(ps: [Direction; 8]) -> u8 {
    let mut result = 0u8;
    let mut dex = 0;
    for x in ps.iter() {
        if let Direction::Input(_) = x {
            result = result | 1u8 >> dex;
        };
        dex += 1;
    }
    result
}

fn read_word(ps: u8) -> [bool; 8] {
    let mut result = [false; 8];
    for ordinal in 0..8 {
        if 0 != ps & (1u8 >> ordinal) {
            result[ordinal] = true
        }
    }
    result
}

fn as_word(ps: [bool; 8]) -> u8 {
    let mut result = 0u8;
    let mut dex = 0;
    for x in ps.iter() {
        result |= if *x { 1u8 >> dex } else { 0u8 };
        dex += 1;
    }
    result
}

pub fn pin_ordinal(p: Pin) -> usize {
    match p {
        Pin::Pin0 => 0,
        Pin::Pin1 => 1,
        Pin::Pin2 => 2,
        Pin::Pin3 => 3,
        Pin::Pin4 => 4,
        Pin::Pin5 => 5,
        Pin::Pin6 => 6,
        Pin::Pin7 => 7,
    }
}

pub fn ordinal_pin(p: usize) -> Option<Pin> {
    match p {
        0 => Some(Pin::Pin0),
        1 => Some(Pin::Pin1),
        2 => Some(Pin::Pin2),
        3 => Some(Pin::Pin3),
        4 => Some(Pin::Pin4),
        5 => Some(Pin::Pin5),
        6 => Some(Pin::Pin6),
        7 => Some(Pin::Pin7),
        _ => None,
    }
}

pub fn bank_pin_to_index(bank: Bank, pin: Pin) -> usize {
    match bank {
        Bank::A => pin_ordinal(pin),
        Bank::B => pin_ordinal(pin) * 2,
    }
}

pub fn index_to_bank_pin(index: usize) -> Result<(Bank, Pin)> {
    let bank = if (index >> 3) & 1 == 0 {
        Bank::A
    } else {
        Bank::B
    };
    match ordinal_pin(index % 8) {
        Some(pin) => Ok((bank, pin)),
        None => Err(Error::OutOfBounds(index)),
    }
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Bank {
    A,
    B,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
struct BankState<T> {
    a: T,
    b: T,
}

/**
 * the last-modified values
 */
#[derive(Debug, PartialEq, Clone, PartialOrd)]
struct State {
    direction: [Direction; 8],
}

const INITIAL_STATE: BankState<State> = BankState {
    a: State {
        direction: [Direction::Input(Pullup::Off); 8],
    },
    b: State {
        direction: [Direction::Input(Pullup::Off); 8],
    },
};

#[derive(Debug, Clone)]
pub struct Mcp23017State {
    state: BankState<State>,
}

impl Mcp23017State {
    pub fn new() -> Self {
        Mcp23017State {
            state: INITIAL_STATE,
        }
    }

    fn state_for_bank(&self, bank: Bank) -> &State {
        match bank {
            Bank::A => &self.state.a,
            Bank::B => &self.state.b,
        }
    }

    fn set_state_for_bank(&mut self, bank: Bank, state: State) {
        match bank {
            Bank::A => self.state.a = state,
            Bank::B => self.state.b = state,
        }
    }

    // Unconditionally writes current value to device
    fn write_gpio_value(
        &self,
        address: Address,
        bank: Bank,
        values: [bool; 8],
        i2c: &I2cBus,
    ) -> Result<()> {
        let register = match bank {
            Bank::A => WRITE_GPIOA_ADDR,
            Bank::B => WRITE_GPIOB_ADDR,
        };
        if cfg!(raspberry_pi) {
            i2c.write(address, register, vec![as_word(values)])
        } else {
            Ok(())
        }
    }

    // Unconditionally reads values from the device and stores in device state
    fn read_gpio_value(&self, address: Address, bank: Bank, i2c: &I2cBus) -> Result<[bool; 8]> {
        let register = match bank {
            Bank::A => READ_GPIOA_ADDR,
            Bank::B => READ_GPIOB_ADDR,
        };

        if cfg!(raspberry_pi) {
            let result = i2c.read(address, register, 1)?;
            Ok(read_word(result[0]))
        } else {
            Ok([true, false, true, true, false, true, true, false])
        }
    }

    // Unconditionally writes current direction to device
    fn write_gpio_dir(&self, address: Address, bank: Bank, i2c: &I2cBus) -> Result<()> {
        let dir = self.state_for_bank(bank).direction;
        let (dir_reg, pullup_reg) = match bank {
            Bank::A => (REGISTER_GPIOA, REGISTER_GPIOA_PULLUP),
            Bank::B => (REGISTER_GPIOB, REGISTER_GPIOB_PULLUP),
        };
        if cfg!(raspberry_pi) {
            i2c.write(address, dir_reg, vec![direction_as_inout_word(dir)])?;
            i2c.write(address, pullup_reg, vec![direction_as_pullup_word(dir)])?;
        }

        Ok(())
    }

    pub fn reset(&mut self, address: Address, i2c: &I2cBus) -> Result<()> {
        self.state = INITIAL_STATE;
        self.write_gpio_dir(address, Bank::A, i2c)?;
        self.write_gpio_dir(address, Bank::B, i2c)?;
        self.write_gpio_value(
            address,
            Bank::A,
            [false, false, false, false, false, false, false, false],
            i2c,
        )?;
        self.write_gpio_value(
            address,
            Bank::B,
            [false, false, false, false, false, false, false, false],
            i2c,
        )?;
        Ok(())
    }

    pub fn set_pin_direction(
        &mut self,
        address: Address,
        bank: Bank,
        pin: Pin,
        direction: Direction,
        i2c: &I2cBus,
    ) -> Result<()> {
        let bank_state = self.state_for_bank(bank);
        let pdex = pin_ordinal(pin);
        if bank_state.direction[pdex] == direction {
            return Ok(());
        }

        let mut new_dir = bank_state.direction;
        new_dir[pdex] = direction;
        let new_state = State {
            direction: new_dir,
            ..*bank_state
        };
        self.set_state_for_bank(bank, new_state);
        self.write_gpio_dir(address, bank, i2c)?;
        Ok(())
    }

    pub fn set_pin(
        &mut self,
        address: Address,
        bank: Bank,
        pin: Pin,
        value: bool,
        i2c: &I2cBus,
    ) -> Result<()> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        if bank_state.direction[pdex] != Direction::Output {
            return Err(Error::InvalidPinDirection);
        }

        let mut current = self.read_gpio_value(address, bank, i2c)?;

        if current[pdex] == value {
            return Ok(());
        }
        current[pdex] = value;
        self.write_gpio_value(address, bank, current, i2c)?;
        Ok(())
    }

    pub fn get_pin_direction(&self, bank: Bank, pin: Pin) -> Result<Direction> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        return Ok(bank_state.direction[pdex]);
    }

    pub fn get_pin(&self, address: Address, bank: Bank, pin: Pin, i2c: &I2cBus) -> Result<bool> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        if let Direction::Output = bank_state.direction[pdex] {
            return Err(Error::InvalidPinDirection);
        }
        let value = self.read_gpio_value(address, bank, i2c)?;
        return Ok(value[pin_ordinal(pin)]);
    }
}
