use crate::session::AppContext;
use juniper::{FieldResult, RootNode};

pub struct Query;

#[juniper::object(Context = AppContext)]
impl Query {
    pub fn active_user(context: &AppContext) -> FieldResult<bool> {
        Ok(false)
    }

    pub fn inputs(context: &AppContext) -> FieldResult<bool> {
        Ok(false)
    }

    pub fn outputs(context: &AppContext) -> FieldResult<bool> {
        Ok(false)
    }
}

pub struct Mutation;

#[juniper::object(Context = AppContext)]
impl Mutation {
    pub fn sign_in(
        context: &AppContext,
        email: String,
        plaintext_password: String,
    ) -> FieldResult<bool> {
        // do a password check, and start a session if ok.
        // return user that was signed in?
        Ok(false)
    }

    pub fn sign_out(context: &AppContext) -> FieldResult<bool> {
        // expire all existing sessions by bumping session count
        Ok(false)
    }
}

pub type Schema = RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation)
}
