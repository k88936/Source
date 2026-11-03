//! Cross-layer DTOs. These are the single source of truth for the Tauri
//! command/event payloads and the generated TypeScript bindings.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, TS)]
        #[ts(export, type = "string")]
        pub struct $name(pub String);

        impl Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                String::deserialize(deserializer).map(Self)
            }
        }

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_type!(AssetId);
id_type!(FolderId);
id_type!(AlbumId);
id_type!(UploadTaskId);

mod assets;
mod errors;
mod index;
mod local;
mod maintenance;
mod params;
mod session;
mod uploads;

pub use assets::*;
pub use errors::*;
pub use index::*;
pub use local::*;
pub use maintenance::*;
pub use params::*;
pub use session::*;
pub use uploads::*;
