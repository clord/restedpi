use crate::app::db::models;
pub use crate::config::types::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use crate::session::AppContext;
use juniper::graphql_object;

use super::dimensioned::Dimensioned;

/**
 * we can write a boolean value to a given device via name
 */
#[derive(Debug, Clone)]
pub struct Output {
    pub data: models::Output,
}

impl Output {
    pub fn name(&self) -> &str {
        self.data.name.as_str()
    }

    pub async fn value(&self, context: &AppContext) -> Dimensioned {
        match context
            .channel()
            .current_output_value(self.data.name.clone())
            .await
        {
            Ok(v) => Dimensioned::from_bool(v),
            Err(e) => Dimensioned::from_error(e.to_string()),
        }
    }
}

#[graphql_object(context = AppContext)]
impl Output {
    pub fn name(&self) -> &str {
        self.data.name.as_str()
    }

    pub async fn device(&self, context: &AppContext) -> Option<crate::app::device::Device> {
        context
            .channel()
            .get_device(self.data.device_id.clone())
            .await
            .ok()
    }

    pub fn device_id(&self) -> &str {
        &self.data.device_id
    }

    pub fn device_output_id(&self) -> i32 {
        self.data.device_output_id
    }

    pub fn active_low(&self) -> bool {
        self.data.active_low
    }

    pub fn automation_script(&self) -> Option<String> {
        self.data.automation_script.clone()
    }

    pub async fn value(&self, context: &AppContext) -> Dimensioned {
        match context
            .channel()
            .current_output_value(self.data.name.clone())
            .await
        {
            Ok(v) => Dimensioned::from_bool(v),
            Err(e) => Dimensioned::from_error(e.to_string()),
        }
    }
}
