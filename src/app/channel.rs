use crate::app::state;
use crate::config;
use crate::error::Result;
use chrono::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use std::time::Duration;
use std::vec::Vec;

/**
 * Supported messages we can send to the app
 */
#[derive(Debug)]
pub enum AppMessage {
    /**
     * run any device-specific reset procedures
     */
    ResetDevice {
        device_id: String,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Return all devices configs
     */
    AllDevices {
        response: oneshot::Sender<
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
     * Return all outputs
     */
    AllOutputs {
        response: oneshot::Sender<HashMap<String, config::Output>>,
    },

    /**
     * Return all inputs
     */
    AllInputs {
        response: oneshot::Sender<HashMap<String, config::Input>>,
    },

    /**
     * Read a set of booleans in a group from a set of inputs
     * result is a vec in same order as input_ids with result of reading each one.
     */
    ReadBooleans {
        input_ids: Vec<String>,
        response: oneshot::Sender<Vec<Result<bool>>>,
    },

    /**
     * Read a single f64 value with unit from an input
     * result is the value read, or an error
     */
    ReadValue {
        input_id: String,
        response: oneshot::Sender<Result<(f64, config::Unit)>>,
    },

    CurrentOutputValue {
        output_id: String,
        response: oneshot::Sender<Result<bool>>,
    },

    /**
     * Read a single boolean value from an input
     * result is the value read, or an error
     */
    ReadBoolean {
        input_id: String,
        response: oneshot::Sender<Result<bool>>,
    },

    /**
     * write a boolean to a given output.
     */
    WriteBoolean {
        output_id: String,
        value: bool,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * add or replace device at a given id
     */
    AddOrReplaceDevice {
        device_id: String,
        config: config::Device,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Remove device at a given id.
     * result is all affected inputs and outputs.
     * any affected inputs or outputs will alsso be removed.
     */
    RemoveDevice {
        device_id: String,
        response: oneshot::Sender<
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
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * remove an output.
     */
    RemoveOutput {
        output_id: String,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Read config of a given device, and also all associated inputs and outputs
     */
    GetDeviceConfig {
        device_id: String,
        response: oneshot::Sender<
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
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Add or replace output.
     */
    AddOrReplaceInput {
        input_id: String,
        input: config::Input,
        response: oneshot::Sender<Result<()>>,
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
        response: oneshot::Sender<Result<DateTime<Local>>>,
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
    sender: mpsc::Sender<AppMessage>,
    users: HashMap<String, String>,
}

impl AppChannel {
    pub async fn terminate(&self) -> Result<()> {
        self.sender.clone().send(AppMessage::Terminate).await?;
        Ok(())
    }

    pub fn hash_for(&self, user: &str) -> Option<&String> {
        self.users.get(user)
    }

    pub async fn set_now(&self) -> Result<()> {
        let time = Local::now();
        Ok(self.sender.clone().send(AppMessage::SetTime { time }).await?)
    }

    pub async fn get_now(&self) -> Result<DateTime<Local>> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::GetTime { response }).await?;
        receiver.await?
    }

    pub async fn reset_device(&self, device_id: String) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::ResetDevice {
            device_id,
            response,
        }).await?;
        receiver.await?
    }

    pub async fn all_outputs(&self) -> Result<HashMap<String, config::Output>> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AllOutputs { response }).await?;
        Ok(receiver.await?)
    }

    pub async fn all_inputs(&self) -> Result<HashMap<String, config::Input>> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AllInputs { response }).await?;
        Ok(receiver.await?)
    }

    pub async fn all_devices(
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
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AllDevices { response }).await?;
        Ok(receiver.await?)
    }

    pub async fn read_booleans(&self, input_ids: Vec<String>) -> Result<Vec<Result<bool>>> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::ReadBooleans {
            response,
            input_ids,
        }).await?;
        Ok(receiver.await?)
    }

    pub async fn read_boolean(&self, input_id: &str) -> Result<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone()
            .send(AppMessage::ReadBoolean { response, input_id: input_id.to_string()}).await?;
        receiver.await?
    }

    pub async fn write_boolean(&self, output_id: String, value: bool) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::WriteBoolean {
            response,
            output_id,
            value,
        }).await?;
        receiver.await?
    }

    pub async fn current_output_value(&self, output_id: &str)->Result<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone()
            .send(AppMessage::CurrentOutputValue { response, output_id: output_id.to_string()}).await?;
        receiver.await?
    }

    pub async fn read_value(&self, input_id: &str) -> Result<(f64, config::Unit)> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone()
            .send(AppMessage::ReadValue { response, input_id : input_id.to_string()}).await?;
        receiver.await?
    }

    pub async fn add_or_replace_device(&self, device_id: String, config: config::Device) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AddOrReplaceDevice {
            response,
            device_id,
            config,
        }).await?;
        receiver.await?
    }

    pub async fn remove_device(
        &self,
        device_id: String,
    ) -> Result<(
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::RemoveDevice {
            response,
            device_id,
        }).await?;
        receiver.await?
    }

    pub async fn remove_input(&self, input_id: String) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone()
            .send(AppMessage::RemoveInput { response, input_id }).await?;
        receiver.await?
    }

    pub async fn remove_output(&self, output_id: String) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::RemoveOutput {
            response,
            output_id,
        }).await?;
        receiver.await?
    }

    pub async fn get_device_config(
        &self,
        device_id: &str,
    ) -> Result<(
        config::Device,
        HashMap<String, config::Input>,
        HashMap<String, config::Output>,
    )> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::GetDeviceConfig {
            response,
            device_id:device_id.to_string(),
        }).await?;
        receiver.await?
    }

    pub async fn add_or_replace_input(&self, input_id: String, input: config::Input) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AddOrReplaceInput {
            response,
            input,
            input_id,
        }).await?;
        receiver.await?
    }

    pub async fn add_or_replace_output(&self, output_id: String, output: config::Output) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender.clone().send(AppMessage::AddOrReplaceOutput {
            response,
            output,
            output_id,
        }).await?;
        receiver.await?
    }
}

/**
 * Given a message and a mut ref to the app, will update app
 *
 * @returns true if channel should terminate
 */
async fn process_message(message: AppMessage, state: &mut state::State) -> bool {
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
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::ReadBoolean { input_id, response } => {
            let result = state.read_input_bool(&input_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::WriteBoolean {
            output_id,
            value,
            response,
        } => {
            let result = state.write_output_bool(&output_id, value);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddOrReplaceDevice {
            device_id,
            config,
            response,
        } => {
            let result = state.add_device(&device_id, config).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::RemoveDevice {
            device_id,
            response,
        } => {
            let result = state.remove_device(&device_id).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AllDevices { response } => {
            let result = state.devices();
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AllOutputs { response } => {
            let result = state.outputs();
            match response.send(result.clone()) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AllInputs { response } => {
            let result = state.inputs();
            match response.send(result.clone()) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::RemoveInput { input_id, response } => {
            let result = state.remove_input(&input_id).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::RemoveOutput {
            output_id,
            response,
        } => {
            let result = state.remove_output(&output_id).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::GetDeviceConfig {
            device_id,
            response,
        } => {
            let result = state.device_config(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
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
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddOrReplaceOutput {
            output_id,
            output,
            response,
        } => {
            let result = state.add_output(&output_id, &output).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddOrReplaceInput {
            input_id,
            input,
            response,
        } => {
            let result = state.add_input(&input_id, &input).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::CurrentOutputValue { output_id, response } => {
            let result = state.read_output_bool(&output_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        },

        AppMessage::ReadValue { input_id, response } => {
            let result = state.read_input_value(&input_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }
        AppMessage::GetTime { response } => {
            let result = Ok(state.current_dt());
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }
        AppMessage::Terminate => should_terminate = true,
    }
    return should_terminate;
}

fn read_item<T: 'static + Send + serde::de::DeserializeOwned + serde::Serialize>(
    path: PathBuf,
) -> Result<(HashMap<String, T>, mpsc::Sender<HashMap<String, T>>)> {
    let cloned_path = path.clone();
    let contents = fs::read_to_string(path).unwrap_or("{}".to_string());
    let config = toml::from_str(&contents).unwrap_or_else(|e| {
        error!("error parsing item: {}", e);
        HashMap::new()
    });

    let (sender, mut receiver) = mpsc::channel::<HashMap<String, T>>(3);

    tokio::spawn(async move { 
        loop {
            match receiver.recv().await {
                Some(config) => match toml::to_string(&config) {
                    Ok(config_as_string) => {
                        fs::write(&cloned_path, config_as_string).expect("failed to write change");
                    }
                    Err(e) => {
                        error!("Failed to encode: {}", e);
                    }
                },
                None => {
                    error!("Failed to write");
                }
            }
        }
    });

    Ok((config, sender))
}

pub fn start_app(
    here: (f64, f64),
    path: &std::path::Path,
    users: HashMap<String, String>,
) -> Result<AppChannel> {
    let (sender, mut receiver) = mpsc::channel::<AppMessage>(10);

    let (devices, devices_change) = read_item(path.join("devices.toml"))?;
    debug!("Devices: {:?}", devices);

    let (inputs, inputs_change) = read_item(path.join("inputs.toml"))?;
    debug!("Inputs: {:?}", inputs);

    let (outputs, outputs_change) = read_item(path.join("outputs.toml"))?;
    debug!("Outputs: {:?}", outputs);

    let mut state = state::new_state(
        here,
        devices,
        devices_change,
        inputs,
        inputs_change,
        outputs,
        outputs_change,
    )?;

    let mut sender_clone = sender.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::delay_for(Duration::from_secs(1)).await;
            match sender_clone.send(AppMessage::SetTime { time: Local::now() }).await {
                Ok(()) => (),
                Err(e) => { 
                    warn!("Error in time loop: {}", e);
                    break;
                }
            };
        }
    });

    tokio::spawn(async move { 
        loop {
            match receiver.recv().await {
                Some(next) => {
                    debug!("processing message: {:?}", &next);
                    if process_message(next, &mut state).await {
                        info!("terminating channel");
                        break;
                    } else {
                        state.emit_automations();
                    }

                    // TODO: Support sending real time change notification by allowing clients to send a sender to us,
                    // which we'll keep in a list and notify each time we get to here, with removal upon error.
                }
                None => {
                    break
                }
            }
        }
    });

    Ok(AppChannel { sender, users })
}
