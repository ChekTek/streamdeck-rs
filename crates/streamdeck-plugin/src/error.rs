use std::fmt;

use serde_json::Error as JsonError;
use thiserror::Error;
use tokio_tungstenite::tungstenite;

/// SDK result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors produced by the Stream Deck plugin SDK.
#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "Unable to establish a connection with Stream Deck, missing command line arguments: {0}"
    )]
    MissingRegistrationArgs(String),

    #[error("Failed to parse Stream Deck registration info at `{path}`: {source}")]
    InvalidRegistrationInfo {
        path: String,
        #[source]
        source: JsonError,
    },

    #[error("Invalid version '{value}': expected {{major}}[.{{minor}}[.{{patch}}[.{{build}}]]]")]
    InvalidVersion { value: String },

    #[error(
        "[ERR_NOT_SUPPORTED]: {feature} requires Stream Deck version {required} or higher, but current version is {current}; please update Stream Deck and the \"Software.MinimumVersion\" in the plugin's manifest to \"{required}\" or higher."
    )]
    NotSupported {
        feature: String,
        required: String,
        current: String,
    },

    #[error(
        "[ERR_NOT_SUPPORTED]: {feature} requires manifest SDK version {required} or higher, but found version {actual}; please update the \"SDKVersion\" in the plugin's manifest to {required} or higher."
    )]
    SdkVersionNotSupported {
        feature: String,
        required: u32,
        actual: u32,
    },

    #[error("The action's UUID cannot be empty.")]
    MissingActionUuid,

    #[error("The action's manifestId was not found within the manifest: {0}")]
    ActionNotInManifest(String),

    #[error("Failed to initialize action; device {0} not found")]
    DeviceNotFound(String),

    #[error("Unable to create KeyAction; source event is not a Keypad")]
    NotAKey,

    #[error("Unable to create DialAction; source event is not an Encoder")]
    NotADial,

    #[error("Stream Deck client has not been initialized")]
    NotInitialized,

    #[error("Not connected to Stream Deck")]
    Disconnected,

    #[error("The request timed out")]
    Timeout,

    #[error("Failed to read manifest.json as the file does not exist.")]
    ManifestMissing,

    #[error("Failed to parse manifest.json: {0}")]
    InvalidManifest(#[source] JsonError),

    #[error("Translations must be a JSON object nested under a property named \"Localization\"")]
    InvalidLocalizations,

    #[error("Failed to serialize command: {0}")]
    Serialize(#[from] JsonError),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Message(String),
}

impl Error {
    pub(crate) fn missing_args(names: &[impl fmt::Display]) -> Self {
        let joined = names
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        Self::MissingRegistrationArgs(joined)
    }
}
