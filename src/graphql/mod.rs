use crate::app::db::models;
use crate::app::device;
use crate::app::AppID;
use crate::session::{authenticate, AppContext};
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};
use rppal::system::DeviceInfo;
use tracing::info;

pub struct Query;

#[graphql_object(Context = AppContext)]
impl Query {
    /// Fetch details about the active user, if any
    pub fn active_user(context: &AppContext) -> FieldResult<Option<String>> {
        Ok(context.session.as_ref().map(|a| a.user.clone()))
    }

    /// Get the name/type of the server
    pub fn server_name(_context: &AppContext) -> FieldResult<String> {
        let device = DeviceInfo::new()?;
        Ok(device.model().to_string())
    }

    /// Current date and time used for computing the state of automations
    pub async fn current_date(context: &AppContext) -> FieldResult<String> {
        let now = context.channel().get_now().await?;
        Ok(now.to_rfc3339())
    }

    /// Retrieve a single device
    pub async fn device(device_id: AppID, context: &AppContext) -> FieldResult<device::Device> {
        let device = context.channel().get_device(device_id).await?;
        Ok(device)
    }

    /// Retrieve all inputs 
    pub async fn inputs(context: &AppContext) -> FieldResult<Vec<crate::app::input::Input>> {
        let devices = context.channel().all_inputs().await?;
        Ok(devices)
    }

    /// Retrieve all outputs
    pub async fn outputs(context: &AppContext) -> FieldResult<Vec<crate::app::output::Output>> {
        let devices = context.channel().all_outputs().await?;
        Ok(devices)
    }

    /// Retrieve all devices
    pub async fn devices(context: &AppContext) -> FieldResult<Vec<device::Device>> {
        let devices = context.channel().all_devices().await?;
        Ok(devices)
    }
}

pub struct Mutation;

#[graphql_object(Context = AppContext)]
impl Mutation {
    /// Generate a new token that can be used to access protected endpoints
    pub async fn sign_in(
        context: &AppContext,
        email: String,
        plaintext_password: String,
    ) -> FieldResult<String> {
        Ok(authenticate(context, &email, &plaintext_password).await?)
    }

    /// Sign out from the system, invalidating all tokens for the active user
    pub fn sign_out(_context: &AppContext) -> FieldResult<bool> {
        // expire all existing sessions by bumping session count
        Ok(false)
    }

    /// Add a new device to the system, which can have inputs and outputs
    pub async fn add_device(
        context: &AppContext,
        model: String,
        name: String,
        description: String,
        disabled: Option<bool>,
    ) -> FieldResult<AppID> {
        let model: crate::app::device::Type = serde_json::from_str(&model)?;
        Ok(context
            .channel()
            .add_device(model, name, description, disabled)
            .await?)
    }

    /// Add an input to a device. This input is a way to read data from a device
    pub async fn add_input(
        context: &AppContext,
        new_input: models::NewInput,
    ) -> FieldResult<AppID> {
        Ok(context.channel().add_input(new_input).await?)
    }

    /// Add an output to a device. Outputs denote ways to send data to a device. this output will permit automations.
    pub async fn add_output(
        context: &AppContext,
        new_output: models::NewOutput,
    ) -> FieldResult<AppID> {
        info!("Adding output {:?}", new_output);
        Ok(context.channel().add_output(new_output).await?)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<AppContext>>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
