use crate::app::db::models;
use crate::app::db::models::UpdateOutput;
use crate::app::device;
use crate::app::input::Input;
use crate::app::output::Output;
use crate::app::AppID;
use crate::error::Error;
use crate::session::{authenticate, AppContext};
use futures::Stream;
use juniper::{graphql_object, graphql_subscription, FieldError, FieldResult, RootNode};
use std::pin::Pin;
use std::time::Duration;

#[cfg(feature = "raspberrypi")]
use rppal::system::DeviceInfo;

use tracing::{debug, info};

pub struct Query;

#[graphql_object(Context = AppContext)]
impl Query {
    /// Fetch details about the active user, if any
    pub fn active_user(context: &AppContext) -> FieldResult<Option<String>> {
        Ok(context.session.as_ref().map(|a| a.user.clone()))
    }

    /// Get the name/type of the server
    pub fn server_name(_context: &AppContext) -> FieldResult<String> {
        #[cfg(not(feature = "raspberrypi"))]
        let device = "NON_RASPBERRYPI";
        #[cfg(feature = "raspberrypi")]
        let device = DeviceInfo::new()?.model();

        Ok(device.to_string())
    }

    pub async fn evaluate_expression(expression: String, context: &AppContext) -> FieldResult<f64> {
        let result = context.channel().evaluate_expression(expression).await?;
        Ok(result)
    }

    pub async fn evaluate_bool_expression(
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

    /// Sign out from the system.
    ///
    /// Note: This API uses stateless JWT tokens, so server-side session invalidation
    /// is not possible. This endpoint verifies the session is valid but always returns
    /// false. Clients should discard their token to complete sign-out.
    pub fn sign_out(context: &AppContext) -> FieldResult<bool> {
        check_session(context)?;
        // JWT tokens are stateless - client must discard token to sign out
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
        check_session(context)?;
        context.channel().write_boolean(output_id, value).await?;
        Ok(true)
    }

    pub async fn update_output(
        context: &AppContext,
        output_id: AppID,
        fields: UpdateOutput,
    ) -> FieldResult<AppID> {
        check_session(context)?;
        info!("Updating output {} with {:?}", output_id, fields);
        let outp = context.channel().update_output(output_id, fields).await?;
        Ok(outp)
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

/// Snapshot of all inputs and outputs at a point in time
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub timestamp: String,
}

#[juniper::graphql_object(Context = AppContext)]
impl StateSnapshot {
    fn inputs(&self) -> &[Input] {
        &self.inputs
    }

    fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

pub struct Subscription;

type StateStream = Pin<Box<dyn Stream<Item = Result<StateSnapshot, FieldError>> + Send>>;

#[graphql_subscription(context = AppContext)]
impl Subscription {
    /// Subscribe to state changes - polls every 3 seconds
    async fn state_updates(context: &AppContext) -> StateStream {
        let channel = context.channel().clone();

        let stream = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(3));

            loop {
                interval.tick().await;

                let inputs = match channel.all_inputs().await {
                    Ok(i) => i,
                    Err(e) => {
                        yield Err(FieldError::new(
                            e.to_string(),
                            juniper::Value::Null,
                        ));
                        continue;
                    }
                };

                let outputs = match channel.all_outputs().await {
                    Ok(o) => o,
                    Err(e) => {
                        yield Err(FieldError::new(
                            e.to_string(),
                            juniper::Value::Null,
                        ));
                        continue;
                    }
                };

                let timestamp = match channel.get_now().await {
                    Ok(t) => t.to_rfc3339(),
                    Err(e) => {
                        yield Err(FieldError::new(
                            e.to_string(),
                            juniper::Value::Null,
                        ));
                        continue;
                    }
                };

                debug!("Subscription emitting state update with {} inputs, {} outputs", inputs.len(), outputs.len());

                yield Ok(StateSnapshot {
                    inputs,
                    outputs,
                    timestamp,
                });
            }
        };

        Box::pin(stream)
    }

    /// Subscribe to input value changes - polls every 3 seconds
    async fn input_updates(
        context: &AppContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<Input>, FieldError>> + Send>> {
        let channel = context.channel().clone();

        let stream = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(3));

            loop {
                interval.tick().await;

                match channel.all_inputs().await {
                    Ok(inputs) => {
                        debug!("Input subscription emitting {} inputs", inputs.len());
                        yield Ok(inputs);
                    }
                    Err(e) => {
                        yield Err(FieldError::new(
                            e.to_string(),
                            juniper::Value::Null,
                        ));
                    }
                }
            }
        };

        Box::pin(stream)
    }

    /// Subscribe to output value changes - polls every 3 seconds
    async fn output_updates(
        context: &AppContext,
    ) -> Pin<Box<dyn Stream<Item = Result<Vec<Output>, FieldError>> + Send>> {
        let channel = context.channel().clone();

        let stream = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(3));

            loop {
                interval.tick().await;

                match channel.all_outputs().await {
                    Ok(outputs) => {
                        debug!("Output subscription emitting {} outputs", outputs.len());
                        yield Ok(outputs);
                    }
                    Err(e) => {
                        yield Err(FieldError::new(
                            e.to_string(),
                            juniper::Value::Null,
                        ));
                    }
                }
            }
        };

        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, Subscription>;

fn check_session(context: &AppContext) -> FieldResult<()> {
    match &context.session {
        None => Err(Error::NotLoggedIn)?,
        Some(_u) => Ok(()),
    }
}

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation, Subscription)
}
