use crate::i2c_bus::error::Error;
use crate::i2c_bus::{I2cBus, Result};
use crate::i2c_bus::i2c_io::Address;

const REGISTER_GPIOA: u8 = 0x00;
const REGISTER_GPIOB: u8 = 0x01;
const REGISTER_GPIOA_PULLUP: u8 = 0x0C;
const REGISTER_GPIOB_PULLUP: u8 = 0x0D;
const READ_GPIOA_ADDR: u8 = 0x12;
const READ_GPIOB_ADDR: u8 = 0x13;
const WRITE_GPIOA_ADDR: u8 = 0x14;
const WRITE_GPIOB_ADDR: u8 = 0x15;

#[derive(PartialEq, Copy, Clone, PartialOrd)]
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

fn pin_ordinal(p: Pin) -> usize {
    match p {
        Pin::Pin0 => 0usize,
        Pin::Pin1 => 1usize,
        Pin::Pin2 => 2usize,
        Pin::Pin3 => 3usize,
        Pin::Pin4 => 4usize,
        Pin::Pin5 => 5usize,
        Pin::Pin6 => 6usize,
        Pin::Pin7 => 7usize,
    }
}

pub fn ordinal_pin(p: usize) -> Option<Pin> {
    match p {
        0usize => Some(Pin::Pin0),
        1usize => Some(Pin::Pin1),
        2usize => Some(Pin::Pin2),
        3usize => Some(Pin::Pin3),
        4usize => Some(Pin::Pin4),
        5usize => Some(Pin::Pin5),
        6usize => Some(Pin::Pin6),
        7usize => Some(Pin::Pin7),
        _ => None,
    }
}

#[derive(PartialEq, Copy, Clone, PartialOrd)]
pub enum Bank {
    A,
    B,
}

#[derive(PartialEq, Copy, Clone, PartialOrd)]
struct BankState<T> {
    a: T,
    b: T,
}

#[derive(PartialEq, Copy, Clone, PartialOrd)]
pub enum Pullup {
    On,
    Off,
}

#[derive(PartialEq, Copy, Clone, PartialOrd)]
pub enum Direction {
    Output,
    Input(Pullup),
}

#[derive(PartialEq, Clone, PartialOrd)]
struct State {
    direction: [Direction; 8],
    value: [bool; 8],
}

const INITIAL_STATE: BankState<State> = BankState {
    a: State {
        direction: [Direction::Input(Pullup::Off); 8],
        value: [false; 8],
    },
    b: State {
        direction: [Direction::Input(Pullup::Off); 8],
        value: [false; 8],
    },
};

#[derive(Clone)]
pub struct Device {
    address: Address,
    state: BankState<State>,
    i2c: I2cBus,
}

impl Device {
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
    fn write_gpio_value(&self, bank: Bank) -> Result<()> {
        let values = self.state_for_bank(bank).value;
        let register = match bank {
            Bank::A => WRITE_GPIOA_ADDR,
            Bank::B => WRITE_GPIOB_ADDR,
        };

        self.i2c.write(self.address, register, vec![as_word(values)])
    }

    // Unconditionally reads values from the device and stores in device state
    fn read_gpio_value(&self, bank: Bank) -> Result<[bool; 8]> {
        let register = match bank {
            Bank::A => READ_GPIOA_ADDR,
            Bank::B => READ_GPIOB_ADDR,
        };

        let result = self.i2c.read(self.address, register, 1)?;
        Ok(read_word(result[0]))
    }

    // Unconditionally writes current direction to device
    fn write_gpio_dir(&self, bank: Bank) -> Result<()> {
        let dir = self.state_for_bank(bank).direction;
        let (dir_reg, pullup_reg) = match bank {
            Bank::A => (REGISTER_GPIOA, REGISTER_GPIOA_PULLUP),
            Bank::B => (REGISTER_GPIOB, REGISTER_GPIOB_PULLUP),
        };
        self.i2c
            .write(self.address, dir_reg, vec![direction_as_inout_word(dir)])?;
        self.i2c.write(
            self.address,
            pullup_reg,
            vec![direction_as_pullup_word(dir)],
        )?;

        Ok(())
    }

    pub fn new(address: u16, i2c: I2cBus) -> Result<Self> {
        let mut device = Device {
            address,
            state: INITIAL_STATE,
            i2c,
        };
        device.reset()?;
        Ok(device)
    }

    pub fn reset(&mut self) -> Result<()> {
        self.state = INITIAL_STATE;
        self.write_gpio_dir(Bank::A)?;
        self.write_gpio_dir(Bank::B)?;
        self.write_gpio_value(Bank::A)?;
        self.write_gpio_value(Bank::B)?;
        Ok(())
    }

    pub fn set_pin_direction(&mut self, bank: Bank, pin: Pin, direction: Direction) -> Result<()> {
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
        self.write_gpio_dir(bank)?;
        Ok(())
    }

    pub fn set_pin(&mut self, bank: Bank, pin: Pin, value: bool) -> Result<()> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        if bank_state.direction[pdex] != Direction::Output {
            return Err(Error::InvalidPinDirection);
        }
        if bank_state.value[pdex] == value {
            return Ok(());
        }
        let mut new_value = bank_state.value;
        new_value[pdex] = value;
        let new_state = State {
            value: new_value,
            ..*bank_state
        };
        self.set_state_for_bank(bank, new_state);
        self.write_gpio_value(bank)?;
        return Ok(());
    }

    pub fn get_pin(&mut self, bank: Bank, pin: Pin) -> Result<bool> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        if let Direction::Output = bank_state.direction[pdex] {
            return Err(Error::InvalidPinDirection);
        }
        let value = self.read_gpio_value(bank)?;
        let new_state = State {
            value,
            ..*bank_state
        };
        self.set_state_for_bank(bank, new_state);
        return Ok(value[pin_ordinal(pin)]);
    }
}
