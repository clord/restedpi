use super::db;
use crate::app::db::models;
use crate::app::device::Device;
use crate::app::input::Input;
use crate::app::output::Output;
use crate::app::{device, state, AppID};
use crate::config::Unit;
use crate::error::Result;
use chrono::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::vec::Vec;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{debug, error, info, instrument, warn};

/**
 * Supported messages we can send to the app
 */
#[derive(Debug)]
pub enum AppMessage {
    /**
     * run any device-specific reset procedures
     */
    ResetDevice {
        device_id: AppID,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Return all devices
     */
    AllDevices {
        response: oneshot::Sender<Result<Vec<Device>>>,
    },

    /**
     * Read a set of booleans in a group from a set of inputs
     * result is a vec in same order as input_ids with result of reading each one.
     */
    ReadBooleans {
        input_ids: Vec<AppID>,
        response: oneshot::Sender<Vec<Result<bool>>>,
    },

    /**
     * Read a single f64 value with unit from an input
     * result is the value read, or an error
     */
    ReadValue {
        input_id: AppID,
        response: oneshot::Sender<Result<(f64, Unit)>>,
    },

    CurrentOutputValue {
        output_id: AppID,
        response: oneshot::Sender<Result<bool>>,
    },

    /**
     * Read a single boolean value from an input
     * result is the value read, or an error
     */
    ReadBoolean {
        input_id: AppID,
        response: oneshot::Sender<Result<bool>>,
    },

    /**
     * write a boolean to a given output.
     */
    WriteBoolean {
        output_id: AppID,
        value: bool,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * add device, returning id
     */
    AddDevice {
        model: device::Type,
        name: String,
        description: String,
        disabled: Option<bool>,
        response: oneshot::Sender<Result<AppID>>,
    },

    /**
     * Remove device at a given id.
     * result is all affected inputs and outputs.
     * any affected inputs or outputs will alsso be removed.
     */
    RemoveDevice {
        device_id: AppID,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * remove an input.
     */
    RemoveInput {
        input_id: AppID,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * remove an output.
     */
    RemoveOutput {
        output_id: AppID,
        response: oneshot::Sender<Result<()>>,
    },

    /**
     * Read config of a given device, and also all associated inputs and outputs
     */
    GetDevice {
        device_id: AppID,
        response: oneshot::Sender<Result<Device>>,
    },

    /**
     * Read inputs for a device
     */
    GetInputsForDevice {
        device_id: AppID,
        response: oneshot::Sender<Result<Vec<Input>>>,
    },

    /**
     * Read outputs for a device
     */
    GetOutputsForDevice {
        device_id: AppID,
        response: oneshot::Sender<Result<Vec<Output>>>,
    },

    /**
     * Add or replace an output.
     */
    AddOutput {
        output: models::NewOutput,
        response: oneshot::Sender<Result<AppID>>,
    },

    /**
     * Add or replace output.
     */
    AddInput {
        input: models::NewInput,
        response: oneshot::Sender<Result<AppID>>,
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
        Ok(self
            .sender
            .clone()
            .send(AppMessage::SetTime { time })
            .await?)
    }

    pub async fn get_now(&self) -> Result<DateTime<Local>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::GetTime { response })
            .await?;
        receiver.await?
    }

    pub async fn reset_device(&self, device_id: AppID) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::ResetDevice {
                device_id,
                response,
            })
            .await?;
        receiver.await?
    }

    pub async fn all_devices(&self) -> Result<Vec<Device>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::AllDevices { response })
            .await?;
        let result = receiver.await?;
        Ok(result?)
    }

    pub async fn read_booleans(&self, input_ids: Vec<AppID>) -> Result<Vec<Result<bool>>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::ReadBooleans {
                response,
                input_ids,
            })
            .await?;
        Ok(receiver.await?)
    }

    pub async fn read_boolean(&self, input_id: AppID) -> Result<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::ReadBoolean { response, input_id })
            .await?;
        receiver.await?
    }

    pub async fn write_boolean(&self, output_id: AppID, value: bool) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::WriteBoolean {
                response,
                output_id,
                value,
            })
            .await?;
        receiver.await?
    }

    pub async fn current_output_value(&self, output_id: AppID) -> Result<bool> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::CurrentOutputValue {
                response,
                output_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn read_value(&self, input_id: AppID) -> Result<(f64, Unit)> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::ReadValue { response, input_id })
            .await?;
        receiver.await?
    }

    pub async fn add_device(
        &self,
        model: crate::app::device::Type,
        name: String,
        description: String,
        disabled: Option<bool>,
    ) -> Result<AppID> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::AddDevice {
                response,
                model,
                name,
                description,
                disabled,
            })
            .await?;
        receiver.await?
    }

    // pub async fn add_or_replace_device(
    //     &self,
    //     device_id: AppID,
    //     config: Device,
    // ) -> Result<()> {
    //     let (response, receiver) = oneshot::channel();
    //     self.sender
    //         .clone()
    //         .send(AppMessage::AddOrReplaceDevice {
    //             response,
    //             device_id,
    //             config,
    //         })
    //         .await?;
    //     receiver.await?
    // }

    pub async fn remove_device(&self, device_id: AppID) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::RemoveDevice {
                response,
                device_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn remove_input(&self, input_id: AppID) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::RemoveInput { response, input_id })
            .await?;
        receiver.await?
    }

    pub async fn remove_output(&self, output_id: AppID) -> Result<()> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::RemoveOutput {
                response,
                output_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn get_device(&self, device_id: AppID) -> Result<Device> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::GetDevice {
                response,
                device_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn get_outputs_for_device(&self, device_id: AppID) -> Result<Vec<Output>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::GetOutputsForDevice {
                response,
                device_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn get_inputs_for_device(&self, device_id: AppID) -> Result<Vec<Input>> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::GetInputsForDevice {
                response,
                device_id,
            })
            .await?;
        receiver.await?
    }

    pub async fn add_input(&self, input: models::NewInput) -> Result<AppID> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::AddInput { response, input })
            .await?;
        receiver.await?
    }

    pub async fn add_output(&self, output: models::NewOutput) -> Result<AppID> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .clone()
            .send(AppMessage::AddOutput { response, output })
            .await?;
        receiver.await?
    }
}

/**
 * Given a message and a mut ref to the app, will update app
 *
 * @returns true if channel should terminate
 */
#[instrument(skip(state))]
async fn process_message(message: AppMessage, state: &mut state::State) -> bool {
    let mut should_terminate = false;
    match message {
        AppMessage::ReadBooleans {
            input_ids,
            response,
        } => {
            let mut result = Vec::new();
            for input_id in &input_ids {
                let r = state.read_input_bool(input_id).await;
                result.push(r);
            }
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::ReadBoolean { input_id, response } => {
            let result = state.read_input_bool(&input_id).await;
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
            let result = state.write_output_bool(&output_id, value).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddDevice {
            model,
            name,
            description,
            disabled,
            response,
        } => {
            let result = state.add_device(model, name, description, disabled).await;
            match response.send(result) {
                Ok(id) => id,
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

        AppMessage::GetDevice {
            device_id,
            response,
        } => {
            let result = state.device(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::GetInputsForDevice {
            device_id,
            response,
        } => {
            let result = state.device_inputs(&device_id);
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::GetOutputsForDevice {
            device_id,
            response,
        } => {
            let result = state.device_outputs(&device_id);
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
            let result = state.reset_device(&device_id).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddOutput { output, response } => {
            let result = state.add_output(&output).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::AddInput { input, response } => {
            let result = state.add_input(&input).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::CurrentOutputValue {
            output_id,
            response,
        } => {
            let result = state.read_output_bool(&output_id).await;
            match response.send(result) {
                Ok(..) => (),
                Err(e) => error!("send failed: {:?}", e),
            };
        }

        AppMessage::ReadValue { input_id, response } => {
            let result = state.read_input_value(&input_id).await;
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

pub async fn start_app(
    here: (f64, f64),
    path: &std::path::Path,
    users: HashMap<String, String>,
) -> Result<AppChannel> {
    let (sender, mut receiver) = mpsc::channel::<AppMessage>(10);

    let db = db::Db::start_db(path)?;

    let mut state = state::new_state(here, db).await?;

    let mut sender_clone = sender.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::delay_for(Duration::from_secs(1)).await;
            match sender_clone
                .send(AppMessage::SetTime { time: Local::now() })
                .await
            {
                Ok(()) => (),
                Err(e) => {
                    warn!("Error in time loop: {}", e);
                    break;
                }
            };
        }
    });

    tokio::spawn(async move {
        let mut last_emit = Instant::now();
        loop {
            match receiver.recv().await {
                Some(next) => {
                    debug!("processing message: {:?}", &next);

                    if process_message(next, &mut state).await {
                        info!("terminating channel");
                        break;
                    } else if last_emit.elapsed().as_millis() > 700 {
                        last_emit = Instant::now();
                        debug!("running automation...");
                        state
                            .emit_automations()
                            .await
                            .expect("emit automations errors");
                    }

                    // TODO: Support sending real time change notification by allowing clients to send a sender to us,
                    // which we'll keep in a list and notify each time we get to here, with removal upon error.
                }
                None => break,
            }
        }
    });

    Ok(AppChannel { sender, users })
}
