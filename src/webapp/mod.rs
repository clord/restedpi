use crate::app;
use crate::error::Error;
use serde_json::json;
use tracing::warn;
use warp::{Rejection, Reply, http::Response, http::StatusCode, reject, reply};

pub mod filters;
mod handlers;
pub mod slugify;

// We have to share the app state since warp uses a thread pool
pub type SharedAppState = app::channel::AppChannel;

// Embed index.html at compile time
const INDEX_HTML: &str = include_str!("../../static/index.html");

pub async fn serve_index() -> Result<impl Reply, Rejection> {
    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(INDEX_HTML)
        .map_err(|_| reject::reject())
}

/// Produce a json-compatible error report
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err.find::<Error>() {
        let code = match err {
            Error::Config(_) => 0x2010,
            Error::DbError(_) => 0x2003,
            Error::TzError(_) => 0x2004,
            Error::PbkError(_) => 0x2204,
            Error::IoError(_) => 0x1000,
            Error::InvalidPinDirection => 0x1001,
            Error::ParseError => 0x1002,
            #[cfg(feature = "raspberrypi")]
            Error::I2cError(_) => 0x1003,
            Error::NonExistant(_) => 0x0010,
            Error::NotUnique(_) => 0x1210,
            Error::OutOfBounds(_) => 0x0011,
            Error::DeviceReadError(_) => 0x0311,
            Error::RecvError(_) => 0x0100,
            Error::SendError(_) => 0x0101,
            Error::StorageError(_) => 0x0102,
            Error::UnitError(_) => 0x0001,
            Error::EncodingError(_) => 0x0002,
            Error::InputNotFound(_) => 0x0003,
            Error::OutputNotFound(_) => 0x0004,
            Error::UserNotFound => 0x0005,
            Error::TokenIssue => 0x0006,
            Error::PasswordIssue => 0x0007,
            Error::NotLoggedIn => 0x0000,
        };

        let message = err.to_string();
        warn!("Client request failed (code {}): {}", code, message);
        let json = json!({ "type": "error", "code": code, "message": message });
        Ok(reply::with_status(
            json.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else if err.is_not_found() {
        let json = json!({ "type": "error", "code": 404, "message": "Not found" });
        Ok(reply::with_status(json.to_string(), StatusCode::NOT_FOUND))
    } else {
        Err(err)
    }
}
