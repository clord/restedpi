use crate::app::device;
use crate::app::AppID;
use crate::error::Error;
use crate::session::{authenticate, AppContext};
use juniper::{graphql_object, EmptySubscription, FieldError, FieldResult, RootNode};
use rppal::system::DeviceInfo;
use std::collections::HashMap;

pub struct Query;

#[graphql_object(Context = AppContext)]
impl Query {
    pub fn active_user(context: &AppContext) -> FieldResult<Option<String>> {
        Ok(context.session.as_ref().map(|a| a.user.clone()))
    }

    pub fn server_name(_context: &AppContext) -> FieldResult<String> {
        let device = DeviceInfo::new()?;
        Ok(device.model().to_string())
    }

    pub async fn current_date(context: &AppContext) -> FieldResult<String> {
        let now = context.channel().get_now().await?;
        Ok(now.to_rfc3339())
    }

    pub async fn device(device_id: AppID, context: &AppContext) -> FieldResult<device::Device> {
        let device = context.channel().get_device(device_id).await?;
        Ok(device)
    }

    pub async fn devices(context: &AppContext) -> FieldResult<Vec<device::Device>> {
        let devices = context.channel().all_devices().await?;
        Ok(devices)
    }
}

pub struct Mutation;

#[graphql_object(Context = AppContext)]
impl Mutation {
    pub async fn sign_in(
        context: &AppContext,
        email: String,
        plaintext_password: String,
    ) -> FieldResult<String> {
        Ok(authenticate(context, &email, &plaintext_password).await?)
    }

    pub fn sign_out(_context: &AppContext) -> FieldResult<bool> {
        // expire all existing sessions by bumping session count
        Ok(false)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<AppContext>>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
