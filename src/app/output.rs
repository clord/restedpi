use crate::app::db::models;
pub use crate::config::parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use crate::session::AppContext;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/**
 * we can write a boolean value to a given device via name
 */
#[derive(Debug, Clone)]
pub struct Output {
    data: models::Output,
}

#[graphql_object(context = AppContext)]
impl Output {
    pub fn name(&self) -> &str {
        self.data.name.as_str()
    }

    pub fn output_id(&self) -> i32 {
        self.data.output_id
    }

    pub fn unit(&self) -> Unit {
        match self.data.unit.as_str() {
            "Boolean" => Unit::Boolean,
            "DegC" => Unit::DegC,
            "KPa" => Unit::KPa,
            _ => Unit::Boolean,
        }
    }

    pub async fn device(&self, context: &AppContext) -> Option<crate::rpi::device::Device> {
        context.channel().get_device(self.data.device_id).await.ok()
    }

    pub fn active_low(&self) -> bool {
        self.data.active_low
    }

    pub fn automation_script(&self) -> Option<String> {
        self.data.automation_script.clone()
    }

    pub async fn value(&self, context: &AppContext) -> FieldResult<bool> {
        Ok(context
            .channel()
            .current_output_value(self.data.output_id)
            .await?)
    }
}
