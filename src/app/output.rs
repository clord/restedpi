use crate::session::AppContext;
use crate::app::db;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
pub use crate::config::parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use crate::app::device::Device;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/**
 * we can write a boolean value to a given device via name
 */
#[derive(Debug, Clone)]
pub struct Output {
    data: db::Output
}

#[graphql_object(context = AppContext)]
impl Output {
    pub fn output_id(&self) -> Option<i32> {
        self.data.output_id
    }

    pub fn name(&self) -> &str {
        self.data.name.as_str()
    }

    pub fn unit(&self) -> Unit {
        self.data.unit
    }

    pub async fn device(&self, context: &AppContext) -> Option<Device> {
        context
            .channel()
            .get_device_config(&self.data.device_id)
            .await
            .ok()
            .map(|(cfg, _, _)| cfg)
    }

    pub fn active_low(&self) -> Option<bool> {
        self.data.active_low
    }

    pub fn on_when(&self) -> Option<String> {
        self.data.automation_script.clone()
    }

    pub async fn value(&self, context: &AppContext) -> FieldResult<bool> {
        match self.data.output_id {
            Some(oid) => Ok(context.channel().current_output_value(oid).await?),
            None => Err(FieldError::from("Value Not Found")),
        }
    }
}
