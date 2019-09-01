use rppal::i2c::I2c;

const REGISTER_GPIOA = 0x00;
const REGISTER_GPIOB = 0x01;
const REGISTER_GPIOA_PULLUP = 0x0C;
const REGISTER_GPIOB_PULLUP = 0x0D;
const READ_GPIOA_ADDR = 0x12
const READ_GPIOB_ADDR = 0x13;
const WRITE_GPIOA_ADDR = 0x14;
const WRITE_GPIOB_ADDR = 0x15;
const BASE_ADDRESS  = 0x20; // if the mcp has all adress lines pulled low

mod mcp23017 {

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
        let result = 0u8;
        for (x in ps.iter()) {
            if let Input(true) = x {
                result = result | 1u8 >> pin_ordinal(x)
            }
        }
    }

    fn direction_as_inout_word(ps: [Direction; 8]) -> u8 {
        let result = 0u8;
        for (x in ps.iter()) {
            if let Input(_) = x {
                result = result | 1u8 >> pin_ordinal(x)
            }
        }
    }

    fn read_word(ps : u8) -> [boolean; 8] {
        let result = [false; 8];
        for ordinal in (0..8) {
            if ps & (1u8 >> ordinal) {
                result[ordinal] = true
            }
        }
        return result
    }

    fn as_word(ps: [boolean; 8]) -> u8 {
        let result = 0u8;
        for (x in ps.iter()) {
            result = result | x ? 1u8 : 0u8 >> pin_ordinal(x)
        }
    }

    fn pin_ordinal(p: Pin) -> u8 {
        match p {
            Pin0 => 0u8
            Pin1 => 1u8
            Pin2 => 2u8
            Pin3 => 3u8
            Pin4 => 4u8
            Pin5 => 5u8
            Pin6 => 6u8
            Pin7 => 7u8
        }
    }

    fn pin_mask(p: Pin) -> u8 {
        1u8 >> pin_ordinal(p)
    }

    pub enum Bank {
        A, B
    }

    struct BankState<T> {
        a: T
        b: T
    }

    pub enum Pullup {
        On, Off
    }

    pub enum Direction {
        Output,
        Input(Pullup)
    }

    struct State {
        direction: [Direction; 8]
        value: [boolean; 8]
    }

    const initial_state = Bank<State> { 
        a: State {
            direction: [Direction.Input(false); 8],
            value: [false; 8]
        }, 
        b: State {
            direction: [Direction.Input(false); 8],
            value: [false; 8]
        }
    }

    pub struct Device {
        address: u8
        mode: Mode
        state: Bank<State>
        bus: I2c
    }

    impl Device {
        fn state_for_bank(&self, bank: Bank) -> &State {
            match bank {
                A => self.state.a
                B => self.state.b
            }
        }
        fn mut_state_for_bank(mut&self, bank: Bank) -> mut&State {
            match bank {
                A => self.state.a
                B => self.state.b
            }
        }

        // Unconditionally writes current value to device
        fn write_gpio_value(&self, bank: Bank) -> Result<()> {
            let values = self.state_for_bank(bank).value;
            let register = match bank {
                A => WRITE_GPIOA_ADDR
                B => WRITE_GPIOB_ADDR
            };
            self.i2c.block_write(register, [as_word(values)])
        }

        // Unconditionally reads values from the device and stores in device state
        fn read_gpio_value(mut&self, bank: Bank) -> Result<()> {
            let values: mut&u8 = self.mut_state_for_bank(bank).value;
            let register = match bank {
                A => READ_GPIOA_ADDR
                B => READ_GPIOB_ADDR
            } 
            let word = [0xFFu8; 1];
            self.i2c.block_read(register, word)?; 
            values = read_word(word);
            Ok(())
        }

        // Unconditionally writes current direction to device
        fn write_gpio_dir(&self, bank: Bank) -> Result<()> {
            let dir = self.state_for_bank(bank).direction;
            let (dir_register, pullup_register) = match bank {
                A => (REGISTER_GPIOA, REGISTER_GPIOA_PULLUP)
                B => (REGISTER_GPIOB, REGISTER_GPIOB_PULLUP)
            }
            self.i2c.block_write(dir_register, [direction_as_inout_word(dir)])?;
            self.i2c.block_write(pullup_register, [direction_as_pullup_word(dir)])?;
            Ok(())
        }

        
        pub fn configure(bus: I2c, address_offset: u8, mode: Mode) -> Device {
            let device = Device {
                address: address_offset + BASE_ADDRESS,
                mode,
                state = initial_state,
                bus
            }
            device.reset();
            device
        }

        pub fn reset(mut &self) -> Result<()> {
            self.state = initial_state;
            self.write_gpio_dir(Bank::A)?;
            self.write_gpio_dir(Bank::B)?;
            self.write_gpio_value(Bank::A)?;
            self.write_gpio_value(Bank::B)?;
            Ok(())
        }

        pub fn set_pin_direction(mut&self, bank: Bank, pin: Pin, direction: Direction) -> Result<()> {
            let mut&bank_state = self.mut_state_for_bank(bank)
            let pdex = pin_ordinal(pin);
            if (bank_state.direction[pdex] == direction) {
                return Ok(());
            }

            bank_state.direction[pdex] = direction;
            self.write_gpio_dir(bank)?;
            Ok(())
        }

        pub fn set_pin(mut&self, bank: Bank, pin: Pin, value: bool) -> Result<()> {
            let pdex = pin_ordinal(pin);
            let mut&bank_state = self.mut_state_for_bank(bank);
            if (bank_state.direction[pdex] != Direction::Output) {
                return Err("Can not set pin value if it is set to input") 
            }
            if (bank_state.value[pdex] == value) {
                return Ok(());
            }
            bank_state.value[pdex] = value;
            self.write_gpio_value(bank);
            return Ok(());
        }

        pub fn read_pin(mut&self, bank: Bank, pin: Pin, value: bool) -> Result<boolean> {
            let pdex = pin_ordinal(pin);
            let mut&bank_state = self.mut_state_for_bank(bank);
            if let Direction::Ouput = bank_state.direction[pdex] {
                return Err("Can not read pin value if it is set to output")
            }
            self.read_gpio_value(bank);
            return Ok(bank_state.value[pin_ordinal(pin)])
        }
    }
}
