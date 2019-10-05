extern crate chrono;

use crate::config::Unit;
use crate::i2c::{bmp085, mcp23017, mcp9808, Result, Sensor, Switch};
use crate::i2c::{
    bus,
    error::Error,
    bus::{Address, I2cBus},
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

    pub fn add_sensor(&mut self, name: String, sensor: Box<dyn Sensor>) {
        self.sensors.insert(name, sensor);
    }

    pub fn add_switch(&mut self, name: String, switch: Box<dyn Switch>) {
        self.switches.insert(name, switch);
    }

    pub fn step(&mut self, action: Action) {
        match action {

            Action::Reset => {
                {
                    for v in self.sensors.values_mut() {
                        v.reset().expect("reset");
                    }
                }
                for v in self.switches.values_mut() {
                    v.reset().expect("reset");
                }
                self.dt = Local::now();
            }

            Action::AddBmp085(n, a, res) => match bmp085::Device::new(a, self.i2c.clone(), res) {
                Ok(d) => self.add_sensor(n, Box::new(d)),
                Err(_) => error!("Failed to add {} named {} at {}", "BMP085", n, a),
            },

            Action::AddMcp9808(n, a) => match mcp9808::Device::new(a, self.i2c.clone()) {
                Ok(d) => self.add_sensor(n, Box::new(d)),
                Err(_) => error!("Failed to add {} named {} at {}", "MCP9808", n, a),
            },

            Action::AddMcp23017(n, a) => match mcp23017::Device::new(a, self.i2c.clone()) {
                Ok(d) => self.add_switch(n, Box::new(d)),
                Err(_) => error!("Failed to add {} named {} at {}", "MCP23017", n, a),
            },

            Action::SetTime(t) => {
                self.dt = t;
            }

            Action::CurrentTime(sender) =>
                sender.send(self.dt).expect("send datetime"),

            Action::SwitchSet(name, pin, value) => {
                if let Some(m) = self.switches.get_mut(&name) {
                    m.write_switch(pin, value).expect("send write switch");
                }
            }

            Action::ReadSensor(name, unit, resp) => {
                if let Some(m) = self.sensors.get(&name) {
                    let result = m.read_sensor(unit);
                    resp.send(result).expect("send read sensor");
                }
                else {
                    resp.send(Err(Error::NonExistant(name))).expect("non-existant send");
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
    AddMcp9808(String, Address),
    AddMcp23017(String, Address),
    AddBmp085(String, Address, bmp085::SamplingMode),
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

    pub fn add_bmp085(&self, name: String, address: Address, res: bmp085::SamplingMode) {
        self.action_sender
            .send(Action::AddBmp085(name, address, res))
            .expect("send add");
    }

    pub fn add_mcp9808(&self, name: String, address: Address) {
        self.action_sender
            .send(Action::AddMcp9808(name, address))
            .expect("send add");
    }

    pub fn add_mcp23017(&self, name: String, address: Address) {
        self.action_sender
            .send(Action::AddMcp23017(name, address))
            .expect("send add");
    }
}

/// Start the main app thread
pub fn start() -> Result<AppState> {
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

        for action in action_receiver {
            state.step(action);
        }
    });

    // start a thread that sends events based on time
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        thread_sender
            .send(Action::SetTime(Local::now()))
            .expect("send");
    });

    Ok(AppState { action_sender })
}
