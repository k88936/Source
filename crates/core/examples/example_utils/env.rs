//! Connection settings read from the environment.

use std::env;
use std::path::PathBuf;

/// `CLOUDREVE_SERVER`, `CLOUDREVE_EMAIL`, `CLOUDREVE_PASSWORD`, the catalog
/// file in `SOURCE_E2E_CATALOG`, and an optional `CLOUDREVE_URI` listing root
/// (default `cloudreve://my/`).
pub struct Settings {
    pub server: String,
    pub email: String,
    pub password: String,
    /// The folder the example lists when argv names none.
    pub root: String,
    /// Where the catalog lives; the use case needs one, listing never writes.
    pub catalog: PathBuf,
}

impl Settings {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            server: env::var("CLOUDREVE_SERVER")?,
            email: env::var("CLOUDREVE_EMAIL")?,
            password: env::var("CLOUDREVE_PASSWORD")?,
            root: env::var("CLOUDREVE_URI").unwrap_or_else(|_| "cloudreve://my/".into()),
            catalog: env::var("SOURCE_E2E_CATALOG")?.into(),
        })
    }
}
