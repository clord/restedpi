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
        Ok(self.sender.send(AppMessage::SetTime{time})?)
    }
    // pub fn ResetDevice {
    //     device_id: String,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * Return all devices configs
    //  */
    // AllDevices {
    //     response: Sender<
    //         HashMap<
    //             String,
    //             (
    //                 config::Device,
    //                 HashMap<String, config::Input>,
    //                 HashMap<String, config::Output>,
    //             ),
    //         >,
    //     >,
    // },

    // /**
    //  * Read a set of booleans in a group from a set of inputs
    //  * result is a vec in same order as input_ids with result of reading each one.
    //  */
    // ReadBooleans {
    //     input_ids: Vec<String>,
    //     response: Sender<Vec<Result<bool>>>,
    // },

    // /**
    //  * Read a single boolean value from an input
    //  * result is the value read, or an error
    //  */
    // ReadBoolean {
    //     input_id: String,
    //     response: Sender<Result<bool>>,
    // },

    // /**
    //  * write a boolean to a given output.
    //  */
    // WriteBoolean {
    //     output_id: String,
    //     value: bool,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * add or replace device at a given id
    //  */
    // AddOrReplaceDevice {
    //     device_id: String,
    //     config: config::Device,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * Remove device at a given id.
    //  * result is all affected inputs and outputs.
    //  * any affected inputs or outputs will alsso be removed.
    //  */
    // RemoveDevice {
    //     device_id: String,
    //     response: Sender<
    //         Result<(
    //             HashMap<String, config::Input>,
    //             HashMap<String, config::Output>,
    //         )>,
    //     >,
    // },

    // /**
    //  * remove an input.
    //  */
    // RemoveInput {
    //     input_id: String,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * remove an output.
    //  */
    // RemoveOutput {
    //     output_id: String,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * Read config of a given device, and also all associated inputs and outputs
    //  */
    // GetDeviceConfig {
    //     device_id: String,
    //     response: Sender<
    //         Result<(
    //             config::Device,
    //             HashMap<String, config::Input>,
    //             HashMap<String, config::Output>,
    //         )>,
    //     >,
    // },

    // /**
    //  * Add or replace an output.
    //  */
    // AddOrReplaceOutput {
    //     output_id: String,
    //     output: config::Output,
    //     response: Sender<Result<()>>,
    // },

    // /**
    //  * Add or replace output.
    //  */
    // AddOrReplaceInput {
    //     input_id: String,
    //     input: config::Input,
    //     response: Sender<Result<()>>,
    // },

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
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::ReadBoolean { input_id, response } => {
            let result = state.read_input_bool(&input_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::WriteBoolean {
            output_id,
            value,
            response,
        } => {
            let result = state.write_output_bool(&output_id, value);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::AddOrReplaceDevice {
            device_id,
            config,
            response,
        } => {
            let result = state.add_device(&device_id, &config);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::RemoveDevice {
            device_id,
            response,
        } => {
            let result = state.remove_device(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::AllDevices { response } => {
            let result = state.devices();
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::RemoveInput { input_id, response } => {
            let result = state.remove_input(&input_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::RemoveOutput {
            output_id,
            response,
        } => {
            let result = state.remove_output(&output_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::GetDeviceConfig {
            device_id,
            response,
        } => {
            let result = state.device_config(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::SetTime { time } => {
            state.set_current_dt(time);
        }

        AppMessage::ResetDevice {
            device_id,
            response,
        } => {
            let result = state.reset_device(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::AddOrReplaceOutput {
            output_id,
            output,
            response,
        } => {
            let result = state.add_output(&output_id, output);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
        }

        AppMessage::AddOrReplaceInput {
            input_id,
            input,
            response,
        } => {
            let result = state.add_input(&input_id, input);
            match response.send(result) {
                Ok(..) => (),
                Err(..) => should_terminate = true,
            }
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
        sender_clone
            .send(AppMessage::SetTime { time: Local::now() })
            .expect("Failed to send!");
    });

    thread::spawn(move || loop {
        let next = receiver.recv().unwrap();
        if process_message(next, &mut state) {
            info!("terminating app");
            break;
        }
        // TODO: Support sending real time change notification by allowing clients to send a sender to us,
        // which we'll keep in a list and notify each time we get to here, with removal upon error.
    });

    Ok(AppChannel { sender })
}
