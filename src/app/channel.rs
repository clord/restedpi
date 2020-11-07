use crate::app::state;
use crate::config;
use crate::i2c::Result;

use chrono::prelude::*;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;
use std::vec::Vec;

/**
 * Supported messages we can send to the app
 */
#[derive(Clone, Debug)]
pub enum AppMessage {
    /**
     * run any device-specific reset procedures
     */
    ResetDevice {
        device_id: String,
        response: Sender<Result<()>>,
    },

    /**
     * Return all devices configs
     */
    AllDevices {
        response: Sender<
            HashMap<
                String,
                (
                    config::Device,
                    HashMap<String, config::Input>,
                    HashMap<String, config::Output>,
                ),
            >,
        >,
    },

    /**
     * Read a set of booleans in a group from a set of inputs
     * result is a vec in same order as input_ids with result of reading each one.
     */
    ReadBooleans {
        input_ids: Vec<String>,
        response: Sender<Vec<Result<bool>>>,
    },

    /**
     * Read a single f64 value with unit from an input
     * result is the value read, or an error
     */
    ReadValue {
        input_id: String,
        response: Sender<Result<(f64, config::Unit)>>,
    },

    /**
     * Read a single boolean value from an input
     * result is the value read, or an error
     */
    ReadBoolean {
        input_id: String,
        response: Sender<Result<bool>>,
    },

    /**
     * write a boolean to a given output.
     */
    WriteBoolean {
        output_id: String,
        value: bool,
        response: Sender<Result<()>>,
    },

    /**
     * add or replace device at a given id
     */
    AddOrReplaceDevice {
        device_id: String,
        config: config::Device,
        response: Sender<Result<()>>,
    },

    /**
     * Remove device at a given id.
     * result is all affected inputs and outputs.
     * any affected inputs or outputs will alsso be removed.
     */
    RemoveDevice {
        device_id: String,
        response: Sender<
            Result<(
                HashMap<String, config::Input>,
                HashMap<String, config::Output>,
            )>,
        >,
    },

    /**
     * remove an input.
     */
    RemoveInput {
        input_id: String,
        response: Sender<Result<()>>,
    },

    /**
     * remove an output.
     */
    RemoveOutput {
        output_id: String,
        response: Sender<Result<()>>,
    },

    /**
     * Read config of a given device, and also all associated inputs and outputs
     */
    GetDeviceConfig {
        device_id: String,
        response: Sender<
            Result<(
                config::Device,
                HashMap<String, config::Input>,
                HashMap<String, config::Output>,
            )>,
        >,
    },

    /**
     * Add or replace an output.
     */
    AddOrReplaceOutput {
        output_id: String,
        output: config::Output,
        response: Sender<Result<()>>,
    },

    /**
     * Add or replace output.
     */
    AddOrReplaceInput {
        input_id: String,
        input: config::Input,
        response: Sender<Result<()>>,
    },

    /**
     * Advance the time of the system to specified value.
     * state machine will update all automated outputs for that given time.
     */
    SetTime { time: DateTime<Local> },

    /**
     * Read current time for the app
     */
    GetTime {
        response: Sender<Result<DateTime<Local>>>,
    },

    /**
     * will gracefully terminate the app channel
     */
    Terminate,
}

/**
 * Represent the application to arbitrary threads.
 * controls central app state safely and without dealing with rust locks!
 */
#[derive(Clone, Debug)]
pub struct AppChannel {
    sender: Sender<AppMessage>,
}

impl AppChannel {
    pub fn terminate(&self) -> Result<()> {
        Ok(self.sender.send(AppMessage::Terminate)?)
    }

    pub fn set_now(&self) -> Result<()> {
        let time = Local::now();
        Ok(self.sender.send(AppMessage::SetTime { time })?)
    }

    pub fn get_now(&self) -> Result<DateTime<Local>> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::GetTime { response })?;
        receiver.recv()?
    }

    pub fn reset_device(&self, device_id: String) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::ResetDevice {
            device_id,
            response,
        })?;
        receiver.recv()?
    }

    pub fn all_devices(
        &self,
    ) -> Result<
        HashMap<
            String,
            (
                config::Device,
                HashMap<String, config::Input>,
                HashMap<String, config::Output>,
            ),
        >,
    > {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::AllDevices { response })?;
        Ok(receiver.recv()?)
    }

    pub fn read_booleans(&self, input_ids: Vec<String>) -> Result<Vec<Result<bool>>> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::ReadBooleans {
            response,
            input_ids,
        })?;
        Ok(receiver.recv()?)
    }

    pub fn read_boolean(&self, input_id: String) -> Result<bool> {
        let (response, receiver) = channel();
        self.sender
            .send(AppMessage::ReadBoolean { response, input_id })?;
        receiver.recv()?
    }

    pub fn write_boolean(&self, output_id: String, value: bool) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::WriteBoolean {
            response,
            output_id,
            value,
        })?;
        receiver.recv()?
    }

    pub fn read_value(&self, input_id: String) -> Result<(f64, config::Unit)> {
        let (response, receiver) = channel();
        self.sender
            .send(AppMessage::ReadValue { response, input_id })?;
        receiver.recv()?
    }

    pub fn add_or_replace_device(&self, device_id: String, config: config::Device) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::AddOrReplaceDevice {
            response,
            device_id,
            config,
        })?;
        receiver.recv()?
    }

    pub fn remove_device(
        &self,
        device_id: String,
    ) -> Result<(
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::RemoveDevice {
            response,
            device_id,
        })?;
        receiver.recv()?
    }

    pub fn remove_input(&self, input_id: String) -> Result<()> {
        let (response, receiver) = channel();
        self.sender
            .send(AppMessage::RemoveInput { response, input_id })?;
        receiver.recv()?
    }

    pub fn remove_output(&self, output_id: String) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::RemoveOutput {
            response,
            output_id,
        })?;
        receiver.recv()?
    }

    pub fn get_device_config(
        &self,
        device_id: String,
    ) -> Result<(
        config::Device,
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::GetDeviceConfig {
            response,
            device_id,
        })?;
        receiver.recv()?
    }

    pub fn add_or_replace_input(&self, input_id: String, input: config::Input) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::AddOrReplaceInput {
            response,
            input,
            input_id,
        })?;
        receiver.recv()?
    }

    pub fn add_or_replace_output(&self, output_id: String, output: config::Output) -> Result<()> {
        let (response, receiver) = channel();
        self.sender.send(AppMessage::AddOrReplaceOutput {
            response,
            output,
            output_id,
        })?;
        receiver.recv()?
    }
}

/**
 * Given a message and a mut ref to the app, will update app
 *
 * @returns true if channel should terminate
 */
fn process_message(message: AppMessage, state: &mut state::State) -> bool {
    let mut should_terminate = false;
    match message {
        AppMessage::ReadBooleans {
            input_ids,
            response,
        } => {
            let mut result = Vec::new();
            for input_id in input_ids {
                let r = state.read_input_bool(&input_id);
                result.push(r);
            }
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::ReadBoolean { input_id, response } => {
            let result = state.read_input_bool(&input_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::WriteBoolean {
            output_id,
            value,
            response,
        } => {
            let result = state.write_output_bool(&output_id, value);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::AddOrReplaceDevice {
            device_id,
            config,
            response,
        } => {
            let result = state.add_device(&device_id, &config);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::RemoveDevice {
            device_id,
            response,
        } => {
            let result = state.remove_device(&device_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::AllDevices { response } => {
            let result = state.devices();
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::RemoveInput { input_id, response } => {
            let result = state.remove_input(&input_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::RemoveOutput {
            output_id,
            response,
        } => {
            let result = state.remove_output(&output_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::GetDeviceConfig {
            device_id,
            response,
        } => {
            let result = state.device_config(&device_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::SetTime { time } => {
            state.set_current_dt(time);
        }

        AppMessage::ResetDevice {
            device_id,
            response,
        } => {
            let result = state.reset_device(&device_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::AddOrReplaceOutput {
            output_id,
            output,
            response,
        } => {
            let result = state.add_output(&output_id, &output);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::AddOrReplaceInput {
            input_id,
            input,
            response,
        } => {
            let result = state.add_input(&input_id, &input);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }

        AppMessage::ReadValue { input_id, response } => {
            let result = state.read_input_value(&input_id);
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }
        AppMessage::GetTime { response } => {
            let result = Ok(state.current_dt());
            thread::spawn(move || match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {}", e),
            });
        }
        AppMessage::Terminate => should_terminate = true,
    }
    return should_terminate;
}

pub fn start_app(config: config::Config) -> Result<AppChannel> {
    let (sender, receiver) = channel::<AppMessage>();
    let mut state = state::new(config)?;
    let sender_clone = sender.clone();

    thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        match sender_clone.send(AppMessage::SetTime { time: Local::now() }) {
            Ok(()) => (),
            Err(e) => error!("Failed to send time! {}", e),
        };
    });

    let app_channel = AppChannel { sender };

    let cloned_app_channel = app_channel.clone();
    thread::spawn(move || loop {
        match receiver.recv() {
            Ok(next) => {
                debug!("processing message: {:?}", &next);
                if process_message(next, &mut state) {
                    info!("terminating channel");
                    break;
                } else {
                    state.emit_automations(&cloned_app_channel);
                }

                // TODO: Support sending real time change notification by allowing clients to send a sender to us,
                // which we'll keep in a list and notify each time we get to here, with removal upon error.
            }
            Err(e) => {
                error!("Failed to recv next message: {}", e);
            }
        }
    });

    Ok(app_channel)
}
