//! Process-global storage for the application secret used to sign session tokens.
//!
//! The secret is resolved once at startup (from `app_secret_path` or the
//! `APP_SECRET` environment variable) and stored in a `OnceLock`. This avoids
//! calling `std::env::set_var`, which is unsound once the async runtime's
//! worker threads exist.

use std::sync::OnceLock;

static APP_SECRET: OnceLock<String> = OnceLock::new();

/// Returned by [`init`] when the secret has already been initialized.
#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyInitialized;

/// Store the application secret for the lifetime of the process.
///
/// May only be called once; subsequent calls fail with [`AlreadyInitialized`]
/// and leave the original value in place.
pub fn init(secret: String) -> Result<(), AlreadyInitialized> {
    APP_SECRET.set(secret).map_err(|_| AlreadyInitialized)
}

/// Fetch the application secret, if one was configured at startup.
#[must_use]
pub fn get() -> Option<&'static str> {
    APP_SECRET.get().map(String::as_str)
}
