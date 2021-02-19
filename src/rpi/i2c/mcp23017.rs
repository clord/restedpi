use super::super::RpiApi;
use super::I2cAddress;
use crate::config::{Dir, Directions};
use crate::error::{Error, Result};
use bit_array::BitArray;
use tracing::debug;

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

fn read_word(ps: u8) -> Bits {
    Bits::from_bytes(&[ps])
}

fn as_word(ps: &Bits) -> u8 {
    *ps.to_bytes().first().expect("Failed to parse bits!")
}

pub fn pin_to_ordinal(p: Pin) -> usize {
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

pub fn ordinal_to_pin(p: usize) -> Pin {
    match p % 8 {
        0 => (Pin::Pin0),
        1 => (Pin::Pin1),
        2 => (Pin::Pin2),
        3 => (Pin::Pin3),
        4 => (Pin::Pin4),
        5 => (Pin::Pin5),
        6 => (Pin::Pin6),
        7 => (Pin::Pin7),
        _ => panic!("p % 8 !E [0..7]"),
    }
}

pub fn bank_pin_to_index(bank: Bank, pin: Pin) -> usize {
    match bank {
        Bank::A => pin_to_ordinal(pin),
        Bank::B => pin_to_ordinal(pin) * 2,
    }
}

pub fn index_to_bank_pin(index: usize) -> (Bank, Pin) {
    let bank = if (index >> 3) & 1 == 0 {
        Bank::A
    } else {
        Bank::B
    };
    (bank, ordinal_to_pin(index))
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
#[derive(Debug, PartialEq, Clone)]
struct State {
    direction: Directions,
    values: Bits,
    initial: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            direction: Directions::new(),
            values: Bits::new(),
            initial: true,
        }
    }
    pub fn get_direction(&self, pin: Pin) -> Dir {
        *self.direction.get(pin_to_ordinal(pin))
    }
    pub fn set_direction(&mut self, pin: Pin, dir: Dir) {
        let dir_loc = self.direction.get_mut(pin_to_ordinal(pin));
        *dir_loc = dir;
    }
    pub fn pin_value(&self, pin: Pin) -> bool {
        self.values.get(7 - pin_to_ordinal(pin)).unwrap_or(false)
    }
    pub fn set_value(&mut self, pin: Pin, value: bool) {
        self.values.set(7 - pin_to_ordinal(pin), value);
    }
    pub fn value_as_word(&self) -> u8 {
        as_word(&self.values)
    }

    pub fn pullup_word(&self) -> u8 {
        let mut ba = Bits::new();
        let mut dex = 0;
        for pinord in 0..7usize {
            let dir = self.direction.get(pinord);
            if let Dir::InWithPD = dir {
                ba.set(7 - dex, true);
            }
            dex += 1;
        }
        as_word(&ba)
    }

    pub fn input_polarity_word(&self) -> u8 {
        let mut ba = Bits::new();
        ba.clear();
        as_word(&ba)
    }

    pub fn inout_word(&self) -> u8 {
        let mut ba = Bits::new();
        let mut dex = 0;
        for pinord in 0..7usize {
            let dir = self.direction.get(pinord);
            if let Dir::InWithPD = dir {
                ba.set(7 - dex, true);
            };
            if let Dir::In = dir {
                ba.set(7 - dex, true);
            };
            dex += 1;
        }
        as_word(&ba)
    }
}

#[derive(Debug, Clone)]
pub struct Mcp23017State {
    state: BankState<State>,
}

impl Mcp23017State {
    pub fn new() -> Self {
        Mcp23017State {
            state: BankState {
                a: State::new(),
                b: State::new(),
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
    async fn write_gpio_value(
        &self,
        address: I2cAddress,
        bank: Bank,
        values: Vec<u8>,
        rapi: &RpiApi,
    ) -> Result<()> {
        let register = match bank {
            Bank::A => WRITE_A,
            Bank::B => WRITE_B,
        };
        debug!("will write: {}: {:?}", address, &values);
        rapi.write_i2c(address, register, values).await
    }

    // Unconditionally reads values from the device and stores in device state
    async fn read_gpio_value(
        &self,
        address: I2cAddress,
        bank: Bank,
        rapi: &RpiApi,
    ) -> Result<Bits> {
        let register = match bank {
            Bank::A => READ_A,
            Bank::B => READ_B,
        };

        let result = rapi.read_i2c(address, register, 1).await?;
        debug!("did read: {}: {:?}", address, result);
        Ok(read_word(result[0]))
    }

    // Unconditionally writes current direction to device
    async fn write_gpio_dir(&self, address: I2cAddress, bank: Bank, rapi: &RpiApi) -> Result<()> {
        let state = self.state_for_bank(bank);
        let (dir_reg, polarity_reg, pullup_reg) = match bank {
            Bank::A => (DIRECTION_A, IN_POLARITY_A, PULLUP_A),
            Bank::B => (DIRECTION_B, IN_POLARITY_B, PULLUP_B),
        };
        rapi.write_i2c(address, dir_reg, vec![state.inout_word()])
            .await?;
        rapi.write_i2c(address, polarity_reg, vec![state.input_polarity_word()])
            .await?;
        rapi.write_i2c(address, pullup_reg, vec![state.pullup_word()])
            .await?;
        Ok(())
    }

    pub async fn reset(&mut self, address: I2cAddress, rapi: &RpiApi) -> Result<()> {
        self.state = BankState {
            a: State::new(),
            b: State::new(),
        };
        self.write_gpio_dir(address, Bank::A, rapi).await?;
        self.write_gpio_dir(address, Bank::B, rapi).await?;
        Ok(())
    }

    pub async fn set_pin_directions(
        &mut self,
        address: I2cAddress,
        bank: Bank,
        directions: &Directions,
        rapi: &RpiApi,
    ) -> Result<()> {
        let bank_state = self.mut_state_for_bank(bank);
        bank_state.direction = *directions;
        self.write_gpio_dir(address, bank, rapi).await?;
        for p in 0..7 {
            self.set_pin(address, bank, ordinal_to_pin(p), false, rapi)
                .await?;
        }
        Ok(())
    }

    pub async fn set_pin_direction(
        &mut self,
        address: I2cAddress,
        bank: Bank,
        pin: Pin,
        direction: Dir,
        rapi: &RpiApi,
    ) -> Result<()> {
        let bank_state = self.mut_state_for_bank(bank);
        if bank_state.get_direction(pin) != direction {
            bank_state.set_direction(pin, direction);
            self.write_gpio_dir(address, bank, rapi).await?;
        }
        Ok(())
    }

    fn mutate_pin(&mut self, bank: Bank, pin: Pin, value: bool) -> (u8, bool) {
        let state = self.mut_state_for_bank(bank);
        let initial = state.initial;
        let old_value = state.value_as_word();
        state.set_value(pin, value);
        state.initial = false;
        let new_value = state.value_as_word();
        (new_value, initial || new_value != old_value)
    }

    pub async fn set_pin(
        &mut self,
        address: I2cAddress,
        bank: Bank,
        pin: Pin,
        value: bool,
        rapi: &RpiApi,
    ) -> Result<()> {
        match self.get_pin_direction(bank, pin) {
            Dir::In => Err(Error::InvalidPinDirection),
            Dir::InWithPD => Err(Error::InvalidPinDirection),
            Dir::OutH => {
                debug!(
                    "set_pin: a:{} b:{:?} p:{:?} <- {}",
                    address, bank, pin, value
                );
                let (new_values, changed) = self.mutate_pin(bank, pin, value);
                if changed {
                    self.write_gpio_value(address, bank, vec![new_values], rapi)
                        .await?;
                }
                Ok(())
            }
            Dir::OutL => {
                debug!(
                    "set_pin: a:{} b:{:?} p:{:?} <- {} (OutL)",
                    address, bank, pin, value
                );
                let (new_values, changed) = self.mutate_pin(bank, pin, !value);
                if changed {
                    self.write_gpio_value(address, bank, vec![new_values], rapi)
                        .await?;
                }
                Ok(())
            }
        }
    }

    pub fn get_pin_direction(&self, bank: Bank, pin: Pin) -> Dir {
        self.state_for_bank(bank).get_direction(pin)
    }

    pub async fn get_pin(
        &self,
        address: I2cAddress,
        bank: Bank,
        pin: Pin,
        rapi: &RpiApi,
    ) -> Result<bool> {
        let state = self.state_for_bank(bank);
        match state.get_direction(pin) {
            Dir::OutL => Ok(!state.pin_value(pin)),
            Dir::OutH => Ok(state.pin_value(pin)),
            Dir::In => {
                let value = self.read_gpio_value(address, bank, rapi).await?;
                return Ok(value[7 - pin_to_ordinal(pin)]);
            }
            Dir::InWithPD => {
                let value = self.read_gpio_value(address, bank, rapi).await?;
                return Ok(value[7 - pin_to_ordinal(pin)]);
            }
        }
    }
}
