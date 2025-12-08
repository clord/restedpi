use crate::schema::{devices, inputs, outputs};
use chrono::prelude::*;
use juniper::{GraphQLInputObject, GraphQLObject};

#[derive(Insertable, Clone, Debug, GraphQLObject)]
#[diesel(table_name = devices)]
pub struct NewDevice {
    pub name: String,
    model: String,
    notes: String,
    disabled: Option<bool>,
}

impl NewDevice {
    pub fn new(
        model: crate::app::device::Type,
        name: String,
        notes: String,
        disabled: Option<bool>,
    ) -> Self {
        // TODO: Transform name to a key that is a valid identifier.
        Self {
            model: serde_json::to_string(&model).unwrap(),
            name,
            notes,
            disabled,
        }
    }
}

/**
 * A device has some inputs and outputs, and connects to physical interfaces like gpio.
 */
#[derive(Queryable, Clone, Debug)]
pub struct Device {
    /// What do we name this particular device for identification?
    pub name: String,

    /// What model of device is this? must be a supported type.
    pub model: String,

    /// information about the device that we might need
    pub notes: String,

    /// If disabled, device will not be considered for certain operations
    pub disabled: bool,

    /// When was this created
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Clone, Debug, GraphQLInputObject)]
#[diesel(table_name = inputs)]
pub struct NewInput {
    pub name: String,
    pub device_id: String,
    pub device_input_id: i32,
}

impl NewInput {
    pub fn new(name: String, device_id: String, device_input_id: i32) -> Self {
        // TODO: Generate a valid identifier
        Self {
            name,
            device_id,
            device_input_id,
        }
    }
}

/// Represent a particular input, meaning a source of information from a device.
#[derive(Queryable, Clone, Debug)]
pub struct Input {
    /// What do we want to call this input
    pub name: String,

    /// The device this input is associated with
    pub device_id: String,

    /// Each device can have multiple inputs and outputs, this is a device-specific index. (pin
    /// number, channel, etc)
    pub device_input_id: i32,

    /// When was this created
    pub created_at: NaiveDateTime,
}

#[derive(Clone, Debug, GraphQLInputObject)]
pub struct UpdateOutput {
    pub device_output_id: Option<i32>,
    pub active_low: Option<bool>,
    pub automation_script: Option<Option<String>>,
}

#[derive(Insertable, Clone, Debug, GraphQLInputObject)]
#[diesel(table_name = outputs)]
pub struct NewOutput {
    pub name: String,
    pub device_id: String,
    pub device_output_id: i32,
    pub active_low: bool,
    pub automation_script: Option<String>,
}

impl NewOutput {
    pub fn new(
        name: String,
        device_id: String,
        device_output_id: i32,
        active_low: bool,
        automation_script: Option<String>,
    ) -> Self {
        Self {
            name,
            device_id,
            device_output_id,
            active_low,
            automation_script,
        }
    }
}

/// Represent a particular output, meaning where we send data to a device
#[derive(Queryable, Clone, Debug)]
pub struct Output {
    /// What do we call this device
    pub name: String,

    /// The device this input is associated with
    pub device_id: String,

    /// Each device can have multiple inputs and outputs, this is a device-specific index. (pin
    /// number, channel, etc)
    pub device_output_id: i32,

    /// is the circuit active_low, and hence needing flips
    pub active_low: bool,

    /// If set to an expression, the system will compute this output every state change and write it to the output
    pub automation_script: Option<String>,

    /// When was this created
    pub created_at: NaiveDateTime,
}
