use serde::Serialize;
use serde_json::{Map, Value};

use super::layout::FeedbackPayload;
use super::types::{Resources, State, Target};

/// Command sent from the plugin to Stream Deck.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum PluginCommand {
    #[serde(rename = "setSettings")]
    SetSettings { context: String, payload: Value },
    #[serde(rename = "getSettings")]
    GetSettings {
        context: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    #[serde(rename = "setGlobalSettings")]
    SetGlobalSettings { context: String, payload: Value },
    #[serde(rename = "getGlobalSettings")]
    GetGlobalSettings {
        context: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    #[serde(rename = "getSecrets")]
    GetSecrets {
        context: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    #[serde(rename = "setResources")]
    SetResources { context: String, payload: Resources },
    #[serde(rename = "getResources")]
    GetResources {
        context: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    #[serde(rename = "openUrl")]
    OpenUrl { payload: OpenUrlPayload },
    #[serde(rename = "logMessage")]
    LogMessage { payload: LogMessagePayload },
    #[serde(rename = "setTitle")]
    SetTitle {
        context: String,
        payload: SetTitlePayload,
    },
    #[serde(rename = "setImage")]
    SetImage {
        context: String,
        payload: SetImagePayload,
    },
    #[serde(rename = "setFeedback")]
    SetFeedback {
        context: String,
        payload: FeedbackPayload,
    },
    #[serde(rename = "setFeedbackLayout")]
    SetFeedbackLayout {
        context: String,
        payload: SetFeedbackLayoutPayload,
    },
    #[serde(rename = "showAlert")]
    ShowAlert { context: String },
    #[serde(rename = "showOk")]
    ShowOk { context: String },
    #[serde(rename = "setState")]
    SetState {
        context: String,
        payload: SetStatePayload,
    },
    #[serde(rename = "setTriggerDescription")]
    SetTriggerDescription {
        context: String,
        payload: TriggerDescription,
    },
    #[serde(rename = "switchToProfile")]
    SwitchToProfile {
        context: String,
        device: String,
        payload: SwitchToProfilePayload,
    },
    #[serde(rename = "sendToPropertyInspector")]
    SendToPropertyInspector { context: String, payload: Value },
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenUrlPayload {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogMessagePayload {
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SetTitlePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Target>,
}

/// Options that define how to render a title.
pub type TitleOptions = SetTitlePayload;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SetImagePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Target>,
}

/// Options that define how to render an image.
pub type ImageOptions = SetImagePayload;

#[derive(Debug, Clone, Serialize)]
pub struct SetFeedbackLayoutPayload {
    pub layout: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetStatePayload {
    pub state: State,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerDescription {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_touch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SwitchToProfilePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

/// Registration handshake sent as soon as the WebSocket opens.
#[derive(Debug, Clone, Serialize)]
pub struct RegisterEvent {
    pub event: String,
    pub uuid: String,
}

/// Helper to convert a serializable map into a JSON object payload.
#[allow(dead_code)]
pub fn to_object(value: impl Serialize) -> Result<Map<String, Value>, serde_json::Error> {
    match serde_json::to_value(value)? {
        Value::Object(map) => Ok(map),
        other => Ok({
            let mut map = Map::new();
            map.insert("value".into(), other);
            map
        }),
    }
}
