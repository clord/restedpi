use crate::app::db::models;
use crate::app::device;
use crate::app::AppID;
use crate::error::Error;
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

    pub async fn evaluate_expression(
        expression: String,
        context: &AppContext,
    ) -> FieldResult<bool> {
        let result = context
            .channel()
            .evaluate_bool_expression(expression)
            .await?;
        Ok(result)
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
    pub fn sign_out(context: &AppContext) -> FieldResult<bool> {
        check_session(context)?;
        // TODO: expire all existing sessions for user by bumping session count
        Ok(false)
    }

    /// Add a new mcp9808 at a given address
    pub async fn add_mcp9808(
        context: &AppContext,
        address: i32,
        name: String,
        description: String,
        disabled: Option<bool>,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        let model = device::Type::MCP9808(device::MCP9808 { address });
        Ok(context
            .channel()
            .add_device(model, name, description, disabled)
            .await?)
    }

    /// Add a new bmp085 at a given address
    pub async fn add_bmp085(
        context: &AppContext,
        address: i32,
        mode: device::SamplingMode,
        name: String,
        description: String,
        disabled: Option<bool>,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        let model = device::Type::BMP085(device::BMP085 { address, mode });
        Ok(context
            .channel()
            .add_device(model, name, description, disabled)
            .await?)
    }

    /// Add an MCP23017 device at the given address.
    pub async fn add_mcp23017(
        context: &AppContext,
        address: i32,
        name: String,
        description: String,
        bank_a: Option<device::InputDirections>,
        bank_b: Option<device::InputDirections>,
        disabled: Option<bool>,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        let model = device::Type::MCP23017(device::MCP23017 {
            address,
            bank_a: bank_a.map_or(device::Directions::new(), |x| x.into()),
            bank_b: bank_b.map_or(device::Directions::new(), |x| x.into()),
        });
        Ok(context
            .channel()
            .add_device(model, name, description, disabled)
            .await?)
    }

    /// Remove the specified device and any inputs or outputs that use it
    pub async fn remove_device(context: &AppContext, device_id: AppID) -> FieldResult<bool> {
        check_session(context)?;
        context.channel().remove_device(device_id).await?;
        Ok(true)
    }

    /// Remove the specified input
    pub async fn remove_input(context: &AppContext, input_id: AppID) -> FieldResult<bool> {
        check_session(context)?;
        context.channel().remove_input(input_id).await?;
        Ok(true)
    }

    /// Remove the specified output
    pub async fn remove_output(context: &AppContext, output_id: AppID) -> FieldResult<bool> {
        check_session(context)?;
        context.channel().remove_output(output_id).await?;
        Ok(true)
    }

    /// Add an input to a device. This input is a way to read data from a device
    pub async fn add_input(
        context: &AppContext,
        new_input: models::NewInput,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        Ok(context.channel().add_input(new_input).await?)
    }

    /// Set the output to a given boolean value
    pub async fn set_output(
        context: &AppContext,
        output_id: AppID,
        value: bool,
    ) -> FieldResult<bool> {
        context.channel().write_boolean(output_id, value).await?;
        Ok(true)
    }

    /// Add an output to a device. Outputs denote ways to send data to a device. this output will permit automations.
    pub async fn add_output(
        context: &AppContext,
        new_output: models::NewOutput,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        info!("Adding output {:?}", new_output);
        Ok(context.channel().add_output(new_output).await?)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<AppContext>>;

fn check_session(context: &AppContext) -> FieldResult<()> {
    match &context.session {
        None => Err(Error::NotLoggedIn)?,
        Some(_u) => Ok(()),
    }
}

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}
