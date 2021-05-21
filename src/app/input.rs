use crate::app::db::models;
use crate::app::device::Device;
use crate::config::types::Unit;
use crate::session::AppContext;
use juniper::{graphql_object, GraphQLObject};
use serde_derive::{Deserialize, Serialize};

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

    pub fn unit(&self) -> crate::config::types::Unit {
        self.db.unit
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
