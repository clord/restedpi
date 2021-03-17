use crate::session::AppContext;
use crate::app::db;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
pub use crate::config::parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use crate::app::device::Device;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, GraphQLObject, Debug, PartialEq, Clone)]
pub struct InputValue {
    pub value: f64,
    pub unit: Unit,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub db: db::Input
}

#[graphql_object(context = AppContext)]
impl Input {
    pub fn name(&self) -> &str {
        self.db.name.as_str()
    }

    pub fn input_id(&self) -> i32 {
        self.db.input_id
    }

    pub fn unit(&self) -> Unit {
        Unit::from_str(self.db.unit)
    }

    pub async fn device(&self, context: &AppContext) -> Option<Device> {
        context
            .channel()
            .get_device_config(&self.db.device_id)
            .await
            .ok()
            .map(|(cfg, _, _)| cfg)
    }

    pub async fn bool_value(&self, context: &AppContext) -> Option<bool> {
        match self.input_id.as_ref() {
            Some(id) => context.channel().read_boolean(id).await.ok(),
            None => None,
        }
    }
    pub async fn value(&self, context: &AppContext) -> Option<InputValue> {
        match self.input_id.as_ref() {
            Some(id) => context
                .channel()
                .read_value(id)
                .await
                .ok()
                .map(|(value, unit)| InputValue { value, unit }),
            None => None,
        }
    }
}

