use crate::app::db::models;
use crate::app::device::Device;
use crate::app::dimensioned::Dimensioned;
use crate::session::AppContext;
use juniper::graphql_object;

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
            .get_device(self.db.device_id.clone())
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

    pub async fn value(&self, context: &AppContext) -> Dimensioned {
        match context.channel().read_value(self.db.name.clone()).await {
            Ok((value, unit)) => Dimensioned::new(unit, value),
            Err(e) => Dimensioned::from_error(e.to_string()),
        }
    }
}
