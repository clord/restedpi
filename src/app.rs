extern crate chrono;

use crate::config;
use crate::config::Config;
use crate::config::Unit;
use crate::i2c::{
    bmp085, bus,
    bus::{Address, I2cBus},
    error::Error,
    mcp23017,
    mcp23017::{Bank, Pin},
    mcp9808, Result, Sensor, Switch,
};
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

// Keep current app state in memory, together with device state
struct State {
    dt: DateTime<Local>,
    sensors: HashMap<String, Box<dyn Sensor>>,
    switches: HashMap<String, Box<dyn Switch>>,
    i2c: I2cBus,
}

impl State {
    pub fn create_mcp23017(&mut self, a: Address) -> Result<mcp23017::Device> {
        mcp23017::Device::new(a, self.i2c.clone())
    }
    pub fn create_bmp085(
        &mut self,
        a: Address,
        mode: bmp085::SamplingMode,
    ) -> Result<bmp085::Device> {
        bmp085::Device::new(a, self.i2c.clone(), mode)
    }
    pub fn create_mcp9808(&mut self, a: Address) -> Result<mcp9808::Device> {
        mcp9808::Device::new(a, self.i2c.clone())
    }

    pub fn add_sensor(&mut self, name: String, sensor: Box<dyn Sensor>) {
        self.sensors.insert(name, sensor);
    }

    pub fn add_switch(&mut self, name: String, switch: Box<dyn Switch>) {
        self.switches.insert(name, switch);
    }

    pub fn reset(&mut self) {
        {
            for v in self.sensors.values_mut() {
                v.reset().expect("reset");
            }
        }
        {
            for v in self.switches.values_mut() {
                v.reset().expect("reset");
            }
        }
        self.dt = Local::now();
    }

    pub fn step(&mut self, action: Action) {
        match action {
            Action::Reset => self.reset(),

            Action::SetTime(t) => {
                self.dt = t;
            }

            Action::CurrentTime(sender) => sender.send(self.dt).expect("send datetime"),

            Action::SwitchSet(name, pin, value) => {
                if let Some(m) = self.switches.get_mut(&name) {
                    m.write_switch(pin, value).expect("send write switch");
                }
            }

            Action::ReadSensor(name, unit, resp) => {
                if let Some(m) = self.sensors.get(&name) {
                    let result = m.read_sensor(unit);
                    resp.send(result).expect("send read sensor");
                } else {
                    resp.send(Err(Error::NonExistant(name)))
                        .expect("non-existant send");
                }
            }

            Action::SwitchToggle(name, pin) => {
                if let Some(m) = self.switches.get_mut(&name) {
                    match m.read_switch(pin) {
                        Ok(cur_value) => self.step(Action::SwitchSet(name, pin, !cur_value)),
                        Err(e) => (),
                    };
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Action {
    Reset,
    CurrentTime(Sender<DateTime<Local>>),
    ReadSensor(String, Unit, Sender<Result<f64>>),
    SetTime(DateTime<Local>),
    SwitchSet(String, usize, bool),
    SwitchToggle(String, usize),
}

#[derive(Clone)]
pub struct AppState {
    action_sender: Sender<Action>,
}

impl AppState {
    pub fn set_switch(&self, name: String, pin: usize, value: bool) {
        self.action_sender
            .send(Action::SwitchSet(name, pin, value))
            .expect("send set");
    }

    pub fn read_sensor(&self, name: String, unit: Unit) -> Result<f64> {
        let (response, port) = channel();
        self.action_sender
            .send(Action::ReadSensor(name, unit, response))
            .expect("send read");
        port.recv()?
    }

    pub fn current_dt(&self) -> DateTime<Local> {
        let (response, port) = channel();
        self.action_sender
            .send(Action::CurrentTime(response))
            .expect("failure asking for time");
        port.recv().expect("failure reading time")
    }

    pub fn set_toggle(&self, name: String, pin: usize) {
        self.action_sender
            .send(Action::SwitchToggle(name, pin))
            .expect("send add");
    }
}

/// Start the main app thread
pub fn start(
    sensor_config: HashMap<String, config::Sensor>,
    switch_config: HashMap<String, config::Switch>,
) -> Result<AppState> {
    let (action_sender, action_receiver) = channel();
    let thread_sender = action_sender.clone();

    // this thread is responsible for running the main state machine
    thread::spawn(move || {
        let i2c = bus::start();
        let switches = HashMap::new();
        let sensors = HashMap::new();
        let mut state = State {
            dt: Local::now(),
            sensors,
            switches,
            i2c,
        };

        for (name, config) in sensor_config.iter() {
            match config.device {
                config::SensorType::BMP085 { address, mode } => {
                    let trans_mode = match mode {
                        config::SamplingMode::UltraLowPower => bmp085::SamplingMode::UltraLowPower,
                        config::SamplingMode::Standard => bmp085::SamplingMode::Standard,
                        config::SamplingMode::HighRes => bmp085::SamplingMode::HighRes,
                        config::SamplingMode::UltraHighRes => bmp085::SamplingMode::UltraHighRes,
                    };
                    info!(
                        "Adding BMP085 sensor named '{}' at i2c address {}",
                        name, address
                    );
                    match state.create_bmp085(address, trans_mode) {
                        Ok(dev) => state.add_sensor(name.to_string(), Box::new(dev)),
                        Err(e) => error!("error adding bmp085: {}", e),
                    };
                }
                config::SensorType::MCP9808 { address } => {
                    info!(
                        "Adding MCP9808 sensor named '{}' at i2c address {}",
                        name, address
                    );
                    match state.create_mcp9808(address) {
                        Ok(dev) => state.add_sensor(name.to_string(), Box::new(dev)),
                        Err(e) => error!("error adding mcp9808: {}", e),
                    };
                }
            }
        }
        for (name, config) in switch_config.iter() {
            match config.device {
                config::SwitchType::MCP23017 {
                    address,
                    ref bank0,
                    ref bank1,
                } => {
                    info!(
                        "Adding MCP23017 switch bank named '{}' at i2c address {}",
                        name, address
                    );
                    match state.create_mcp23017(address) {
                        Ok(dev) => {
                            for (bankcfg, bankname) in [(bank0, Bank::A), (bank1, Bank::B)].iter() {
                                for pin in [
                                    Pin::Pin0,
                                    Pin::Pin1,
                                    Pin::Pin2,
                                    Pin::Pin3,
                                    Pin::Pin4,
                                    Pin::Pin5,
                                    Pin::Pin6,
                                    Pin::Pin7,
                                ]
                                .iter()
                                {
                                    // if (bankcfg[ordinal(pin)]) {
                                    // set up
                                    // }
                                }
                            }
                            state.add_switch(name.to_string(), Box::new(dev));
                        }
                        Err(e) => error!("error adding mcp23017: {}", e),
                    };
                }
            }
        }

        for action in action_receiver {
            state.step(action);
        }
    });

    // start a thread that sends events based on time
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        thread_sender
            .send(Action::SetTime(Local::now()))
            .expect("send");
    });

    Ok(AppState { action_sender })
}
