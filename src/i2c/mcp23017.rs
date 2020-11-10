use crate::config::Dir;
use crate::i2c::bus::{Address, I2cBus};
use crate::i2c::{error::Error, Result};
use bit_array::BitArray;

type Bits = BitArray<u32, typenum::U8>;

const DIRECTION_A: u8 = 0x00;
const DIRECTION_B: u8 = 0x01;
const IN_POLARITY_A: u8 = 0x02;
const IN_POLARITY_B: u8 = 0x03;
const PULLUP_A: u8 = 0x0C;
const PULLUP_B: u8 = 0x0D;
const READ_A: u8 = 0x12;
const READ_B: u8 = 0x13;
const WRITE_A: u8 = 0x14;
const WRITE_B: u8 = 0x15;

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

fn direction_as_pullup_word(ps: [Dir; 8]) -> u8 {
    let mut ba = Bits::new();
    let mut dex = 0;
    for x in ps.iter() {
        if let Dir::In(true) = x {
            ba.set(dex, true);
        }
        dex += 1;
    }
    as_word(&ba)
}

fn direction_as_input_polarity_word(_ps: [Dir; 8]) -> u8 {
    let mut ba = Bits::new();
    ba.clear();
    as_word(&ba)
}

fn direction_as_inout_word(ps: [Dir; 8]) -> u8 {
    let mut ba = Bits::new();
    let mut dex = 0;
    for x in ps.iter() {
        if let Dir::In(_) = x {
            ba.set(dex, true);
        };
        dex += 1;
    }
    as_word(&ba)
}

fn read_word(ps: u8) -> Bits {
    Bits::from_bytes(&[ps])
}

fn as_word(ps: &Bits) -> u8 {
    *ps.to_bytes().first().expect("Failed to parse bits!")
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
    direction: [Dir; 8],
    values: Bits,
}

#[derive(Debug, Clone)]
pub struct Mcp23017State {
    state: BankState<State>,
}

impl Mcp23017State {
    pub fn new() -> Self {
        Mcp23017State {
            state: BankState {
                a: State {
                    direction: [Dir::In(false); 8],
                    values: Bits::new(),
                },
                b: State {
                    direction: [Dir::In(false); 8],
                    values: Bits::new(),
                },
            },
        }
    }

    fn state_for_bank(&self, bank: Bank) -> &State {
        match bank {
            Bank::A => &self.state.a,
            Bank::B => &self.state.b,
        }
    }

    fn mut_state_for_bank(&mut self, bank: Bank) -> &mut State {
        match bank {
            Bank::A => &mut self.state.a,
            Bank::B => &mut self.state.b,
        }
    }

    // Unconditionally writes current value to device
    fn write_gpio_value(
        &self,
        address: Address,
        bank: Bank,
        values: Vec<u8>,
        i2c: &I2cBus,
    ) -> Result<()> {
        let register = match bank {
            Bank::A => WRITE_A,
            Bank::B => WRITE_B,
        };
        debug!("will write: {}: {:?}", address, &values);
        i2c.write(address, register, values)
    }

    // Unconditionally reads values from the device and stores in device state
    fn read_gpio_value(&self, address: Address, bank: Bank, i2c: &I2cBus) -> Result<Bits> {
        let register = match bank {
            Bank::A => READ_A,
            Bank::B => READ_B,
        };

        let result = i2c.read(address, register, 1)?;
        debug!("did read: {}: {:?}", address, result);
        Ok(read_word(result[0]))
    }

    // Unconditionally writes current direction to device
    fn write_gpio_dir(&self, address: Address, bank: Bank, i2c: &I2cBus) -> Result<()> {
        let dir = self.state_for_bank(bank).direction;
        let (dir_reg, polarity_reg, pullup_reg) = match bank {
            Bank::A => (DIRECTION_A, IN_POLARITY_A, PULLUP_A),
            Bank::B => (DIRECTION_B, IN_POLARITY_B, PULLUP_B),
        };
        i2c.write(address, dir_reg, vec![direction_as_inout_word(dir)])?;
        i2c.write(address, polarity_reg, vec![direction_as_input_polarity_word(dir)])?;
        i2c.write(address, pullup_reg, vec![direction_as_pullup_word(dir)])?;

        Ok(())
    }

    fn init_pin_value(&self, pin: Pin, bank: Bank) -> bool {
        let bank_state = self.state_for_bank(bank);
        let pdex = pin_ordinal(pin);
        bank_state.direction[pdex] == Dir::OutL
    }

    pub fn reset(&mut self, address: Address, i2c: &I2cBus) -> Result<()> {
        self.state = BankState {
            a: State {
                direction: [Dir::In(false); 8],
                values: Bits::new(),
            },
            b: State {
                direction: [Dir::In(false); 8],
                values: Bits::new(),
            },
        };
        self.write_gpio_dir(address, Bank::A, i2c)?;
        self.write_gpio_dir(address, Bank::B, i2c)?;
        for bank in [Bank::A, Bank::B].iter() {
            let mut bits = Bits::from_elem(false);
            for pin_ord in 0..7 {
                if let Some(pin) = ordinal_pin(pin_ord) {
                    bits.set(pin_ord, self.init_pin_value(pin, *bank))
                }
            }
            self.write_gpio_value(address, *bank, vec![as_word(&bits.clone())], i2c)?;
        }
        Ok(())
    }

    pub fn set_pin_directions(
        &mut self,
        address: Address,
        bank: Bank,
        directions: &[Dir],
        i2c: &I2cBus,
    ) -> Result<()> {
        let bank_state = self.mut_state_for_bank(bank);
        let mut i = 0;
        for dir in directions {
            bank_state.direction[i] = *dir;
            i += 1;
        }
        self.write_gpio_dir(address, bank, i2c)?;
        Ok(())
    }

    pub fn set_pin_direction(
        &mut self,
        address: Address,
        bank: Bank,
        pin: Pin,
        direction: Dir,
        i2c: &I2cBus,
    ) -> Result<()> {
        let bank_state = self.mut_state_for_bank(bank);
        let pdex = pin_ordinal(pin);
        if bank_state.direction[pdex] == direction {
            return Ok(());
        }
        bank_state.direction[pdex] = direction;
        self.write_gpio_dir(address, bank, i2c)?;
        Ok(())
    }

    fn mutate_pin(&mut self, bank: Bank, pin: Pin, value: bool) -> u8 {
        let pdex = pin_ordinal(pin);
        let bank_state = self.mut_state_for_bank(bank);
        bank_state.values.set(pdex, value);
        as_word(&bank_state.values)
    }

    pub fn set_pin(
        &mut self,
        address: Address,
        bank: Bank,
        pin: Pin,
        value: bool,
        i2c: &I2cBus,
    ) -> Result<()> {
        match self.get_pin_direction(bank, pin) {
            Dir::In(..) => Err(Error::InvalidPinDirection),
            Dir::OutH => {
                debug!(
                    "set_pin: a:{} b:{:?} p:{:?} <- {}",
                    address, bank, pin, value
                );
                let new_values = self.mutate_pin(bank, pin, value);
                self.write_gpio_value(address, bank, vec![new_values], i2c)?;
                Ok(())
            }
            Dir::OutL => {
                debug!(
                    "set_pin: a:{} b:{:?} p:{:?} <- {} (OutL)",
                    address, bank, pin, !value
                );
                let new_values = self.mutate_pin(bank, pin, !value);
                self.write_gpio_value(address, bank, vec![new_values], i2c)?;
                Ok(())
            }
        }
    }

    pub fn get_pin_direction(&self, bank: Bank, pin: Pin) -> Dir {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        return bank_state.direction[pdex];
    }

    pub fn get_pin(&self, address: Address, bank: Bank, pin: Pin, i2c: &I2cBus) -> Result<bool> {
        let pdex = pin_ordinal(pin);
        let bank_state = self.state_for_bank(bank);
        match bank_state.direction[pdex] {
            Dir::OutL => Ok(!bank_state.values[pdex]),
            Dir::OutH => Ok(bank_state.values[pdex]),
            Dir::In(..) => {
                let value = self.read_gpio_value(address, bank, i2c)?;
                return Ok(value[pin_ordinal(pin)]);
            }
        }
    }
}
