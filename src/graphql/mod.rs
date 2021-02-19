use crate::config::{Input, Output};
use crate::error::Error;
use crate::session::{authenticate, AppContext};
use juniper::{graphql_object, EmptySubscription, FieldError, FieldResult, RootNode};
use rppal::system::DeviceInfo;

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

    pub async fn inputs(context: &AppContext) -> FieldResult<Vec<String>> {
        let inputs = context.channel().all_inputs().await?;
        Ok(inputs.keys().map(|x| x.to_owned()).collect())
    }

    pub async fn outputs(context: &AppContext) -> FieldResult<Vec<String>> {
        let outputs = context.channel().all_outputs().await?;
        Ok(outputs.keys().map(|x| x.to_owned()).collect())
    }

    pub async fn input(context: &AppContext, id: String) -> FieldResult<Input> {
        let inputs = context.channel().all_inputs().await?;
        match inputs.get(&id) {
            Some(input) => {
                let mut cloned = input.clone();
                cloned.input_id = Some(id);
                Ok(cloned)
            }
            None => Err(FieldError::from(Error::InputNotFound(id))),
        }
    }

    pub async fn output(context: &AppContext, id: String) -> FieldResult<Output> {
        let outputs = context.channel().all_outputs().await?;
        match outputs.get(&id) {
            Some(output) => {
                let mut cloned = output.clone();
                cloned.output_id = Some(id);
                Ok(cloned)
            }
            None => Err(FieldError::from(Error::OutputNotFound(id))),
        }
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
