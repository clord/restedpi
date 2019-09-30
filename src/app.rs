extern crate chrono;

use crate::config::Unit;
use crate::i2c::{bmp085, mcp23017, mcp9808, Result, Sensor, Switch};
use crate::i2c::{
    bus,
    bus::{Address, I2cBus},
};
use chrono::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
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
            Action::Time(t) => {
                // TODO: If time satisfies certain constraints, do things
                self.dt = Local::now();
            },
            Action::SwitchSet(name, pin, value) => {
                if let Some(m) = self.switches.get_mut(&name) {
                    m.write_switch(pin, value);
                }
            },
            Action::ReadSensor(name, unit, resp) => {
                if let Some(m) = self.sensors.get(&name) {
                    let result = m.read_sensor(unit);
                    resp.send(result);
                }
            },
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
enum Action {
    Reset,
    ReadSensor(String, Unit, Sender<Result<f64>>),
    Time(DateTime<Local>),
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
        self.action_sender.send(Action::SwitchSet(name, pin, value));
    }

    pub fn read_sensor(&self, name: String, unit: Unit) -> Result<f64> {
        let (response, port) = channel();
        self.action_sender.send(Action::ReadSensor(name, unit, response));
        port.recv()?
    }

    pub fn set_toggle(&self, name: String, pin: usize) {
        self.action_sender.send(Action::SwitchToggle(name, pin));
    }

    pub fn add_bmp085(&self, name: String, address: Address, res: bmp085::SamplingMode) {
        self.action_sender
            .send(Action::AddBmp085(name, address, res));
    }

    pub fn add_mcp9808(&self, name: String, address: Address) {
        self.action_sender.send(Action::AddMcp9808(name, address));
    }

    pub fn add_mcp23017(&self, name: String, address: Address) {
        self.action_sender.send(Action::AddMcp23017(name, address));
    }
}

/// Start the app
pub fn start() -> Result<AppState> {
    let (action_sender, action_receiver) = channel();
    let thread_sender = action_sender.clone();

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
        // let d1 =  mcp9808::Device::new(0x18u16, bus.clone())?;
        // sensors.insert(String::from("sensor1"), Rc::new(d1) );
        // let d2 =  mcp9808::Device::new(0x19u16, bus.clone())?;
        // sensors.insert(String::from("sensor2"), Rc::new(d2) );
        // let d3 =  mcp9808::Device::new(0x1au16, bus.clone())?;
        // sensors.insert(String::from("sensor3"), Rc::new(d3) );
        // let d4 =  bmp085::Device::new(0x77u16, bus.clone(), bmp085::SamplingMode::HighRes)?;
        // sensors.insert(String::from("sensor4"), Rc::new(d4) );
        // let d5 =  mcp23017::Device::new(0x20u16, bus.clone())?;
        // switches.insert(String::from("switchbank"), Rc::new(d5) );

        for action in action_receiver {
            state.step(action);
        }
    });

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(10));
        thread_sender.send(Action::Time(Local::now()));
    });

    Ok(AppState { action_sender })
}
