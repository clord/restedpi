use crate::app::db::models;
use crate::app::device::Device;
pub use crate::config::parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use crate::session::AppContext;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
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
    pub db: models::Input,
}

#[graphql_object(context = AppContext)]
impl Input {
    pub fn name(&self) -> &str {
        self.db.name.as_str()
    }

    pub fn unit(&self) -> crate::config::Unit {
        serde_json::from_str(&self.db.unit).unwrap()
    }

    pub async fn device(&self, context: &AppContext) -> Option<Device> {
        context
            .channel()
            .get_device(self.db.name.clone())
            .await
            .ok()
    }

    pub async fn bool_value(&self, context: &AppContext) -> Option<bool> {
        context
            .channel()
            .read_boolean(self.db.name.clone())
            .await
            .ok()
    }

    pub async fn value(&self, context: &AppContext) -> Option<InputValue> {
        context
            .channel()
            .read_value(self.db.name.clone())
            .await
            .ok()
            .map(|(value, unit)| InputValue { value, unit })
    }
}
