use serde::Deserialize;
use serde_json::{Map, Value};

use super::types::{Controller, Coordinates, DeviceInfo, State};

/// Events received by the plugin from Stream Deck.
///
/// Unknown `event` names are preserved as [`PluginEvent::Unknown`] so a new Stream Deck
/// event cannot fail inbound parsing or drop the WebSocket frame.
#[derive(Debug, Clone)]
pub enum PluginEvent {
    ApplicationDidLaunch {
        payload: ApplicationPayload,
    },
    ApplicationDidTerminate {
        payload: ApplicationPayload,
    },
    DeviceDidChange {
        device: String,
        device_info: DeviceInfo,
    },
    DeviceDidConnect {
        device: String,
        device_info: DeviceInfo,
    },
    DeviceDidDisconnect {
        device: String,
    },
    DialDown(ActionMessage<EncoderPayload>),
    DialUp(ActionMessage<EncoderPayload>),
    DialRotate(ActionMessage<DialRotatePayload>),
    DidReceiveDeepLink {
        payload: DeepLinkPayload,
    },
    DidReceiveGlobalSettings {
        id: Option<String>,
        payload: SettingsOnlyPayload,
    },
    DidReceiveSecrets {
        id: Option<String>,
        payload: SecretsPayload,
    },
    DidReceiveSettings(ActionMessageWithId<AppearPayload>),
    DidReceiveResources(ActionMessageWithId<AppearPayload>),
    KeyDown(ActionMessage<KeyGesturePayload>),
    KeyUp(ActionMessage<KeyGesturePayload>),
    PropertyInspectorDidAppear(ActionIdentifier),
    PropertyInspectorDidDisappear(ActionIdentifier),
    SendToPlugin {
        action: String,
        context: String,
        payload: Value,
    },
    SystemDidWakeUp,
    TitleParametersDidChange(ActionMessage<TitleParametersPayload>),
    TouchTap(ActionMessage<TouchTapPayload>),
    WillAppear(ActionMessage<AppearPayload>),
    WillDisappear(ActionMessage<AppearPayload>),
    /// Event name this SDK does not model yet. The original JSON is preserved.
    Unknown(Value),
}

/// Wire form of [`PluginEvent`] without the unknown catch-all. Known event names
/// still fail closed when their payload is malformed.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event")]
enum KnownPluginEvent {
    #[serde(rename = "applicationDidLaunch")]
    ApplicationDidLaunch { payload: ApplicationPayload },
    #[serde(rename = "applicationDidTerminate")]
    ApplicationDidTerminate { payload: ApplicationPayload },
    #[serde(rename = "deviceDidChange")]
    DeviceDidChange {
        device: String,
        #[serde(rename = "deviceInfo")]
        device_info: DeviceInfo,
    },
    #[serde(rename = "deviceDidConnect")]
    DeviceDidConnect {
        device: String,
        #[serde(rename = "deviceInfo")]
        device_info: DeviceInfo,
    },
    #[serde(rename = "deviceDidDisconnect")]
    DeviceDidDisconnect { device: String },
    #[serde(rename = "dialDown")]
    DialDown(ActionMessage<EncoderPayload>),
    #[serde(rename = "dialUp")]
    DialUp(ActionMessage<EncoderPayload>),
    #[serde(rename = "dialRotate")]
    DialRotate(ActionMessage<DialRotatePayload>),
    #[serde(rename = "didReceiveDeepLink")]
    DidReceiveDeepLink { payload: DeepLinkPayload },
    #[serde(rename = "didReceiveGlobalSettings")]
    DidReceiveGlobalSettings {
        #[serde(default)]
        id: Option<String>,
        payload: SettingsOnlyPayload,
    },
    #[serde(rename = "didReceiveSecrets")]
    DidReceiveSecrets {
        #[serde(default)]
        id: Option<String>,
        payload: SecretsPayload,
    },
    #[serde(rename = "didReceiveSettings")]
    DidReceiveSettings(ActionMessageWithId<AppearPayload>),
    #[serde(rename = "didReceiveResources")]
    DidReceiveResources(ActionMessageWithId<AppearPayload>),
    #[serde(rename = "keyDown")]
    KeyDown(ActionMessage<KeyGesturePayload>),
    #[serde(rename = "keyUp")]
    KeyUp(ActionMessage<KeyGesturePayload>),
    #[serde(rename = "propertyInspectorDidAppear")]
    PropertyInspectorDidAppear(ActionIdentifier),
    #[serde(rename = "propertyInspectorDidDisappear")]
    PropertyInspectorDidDisappear(ActionIdentifier),
    #[serde(rename = "sendToPlugin")]
    SendToPlugin {
        action: String,
        context: String,
        payload: Value,
    },
    #[serde(rename = "systemDidWakeUp")]
    SystemDidWakeUp,
    #[serde(rename = "titleParametersDidChange")]
    TitleParametersDidChange(ActionMessage<TitleParametersPayload>),
    #[serde(rename = "touchTap")]
    TouchTap(ActionMessage<TouchTapPayload>),
    #[serde(rename = "willAppear")]
    WillAppear(ActionMessage<AppearPayload>),
    #[serde(rename = "willDisappear")]
    WillDisappear(ActionMessage<AppearPayload>),
}

impl From<KnownPluginEvent> for PluginEvent {
    fn from(event: KnownPluginEvent) -> Self {
        match event {
            KnownPluginEvent::ApplicationDidLaunch { payload } => {
                Self::ApplicationDidLaunch { payload }
            }
            KnownPluginEvent::ApplicationDidTerminate { payload } => {
                Self::ApplicationDidTerminate { payload }
            }
            KnownPluginEvent::DeviceDidChange {
                device,
                device_info,
            } => Self::DeviceDidChange {
                device,
                device_info,
            },
            KnownPluginEvent::DeviceDidConnect {
                device,
                device_info,
            } => Self::DeviceDidConnect {
                device,
                device_info,
            },
            KnownPluginEvent::DeviceDidDisconnect { device } => {
                Self::DeviceDidDisconnect { device }
            }
            KnownPluginEvent::DialDown(m) => Self::DialDown(m),
            KnownPluginEvent::DialUp(m) => Self::DialUp(m),
            KnownPluginEvent::DialRotate(m) => Self::DialRotate(m),
            KnownPluginEvent::DidReceiveDeepLink { payload } => {
                Self::DidReceiveDeepLink { payload }
            }
            KnownPluginEvent::DidReceiveGlobalSettings { id, payload } => {
                Self::DidReceiveGlobalSettings { id, payload }
            }
            KnownPluginEvent::DidReceiveSecrets { id, payload } => {
                Self::DidReceiveSecrets { id, payload }
            }
            KnownPluginEvent::DidReceiveSettings(m) => Self::DidReceiveSettings(m),
            KnownPluginEvent::DidReceiveResources(m) => Self::DidReceiveResources(m),
            KnownPluginEvent::KeyDown(m) => Self::KeyDown(m),
            KnownPluginEvent::KeyUp(m) => Self::KeyUp(m),
            KnownPluginEvent::PropertyInspectorDidAppear(m) => Self::PropertyInspectorDidAppear(m),
            KnownPluginEvent::PropertyInspectorDidDisappear(m) => {
                Self::PropertyInspectorDidDisappear(m)
            }
            KnownPluginEvent::SendToPlugin {
                action,
                context,
                payload,
            } => Self::SendToPlugin {
                action,
                context,
                payload,
            },
            KnownPluginEvent::SystemDidWakeUp => Self::SystemDidWakeUp,
            KnownPluginEvent::TitleParametersDidChange(m) => Self::TitleParametersDidChange(m),
            KnownPluginEvent::TouchTap(m) => Self::TouchTap(m),
            KnownPluginEvent::WillAppear(m) => Self::WillAppear(m),
            KnownPluginEvent::WillDisappear(m) => Self::WillDisappear(m),
        }
    }
}

impl<'de> Deserialize<'de> for PluginEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        parse_plugin_event(value).map_err(serde::de::Error::custom)
    }
}

fn parse_plugin_event(value: Value) -> Result<PluginEvent, serde_json::Error> {
    match value.get("event").and_then(Value::as_str) {
        Some(name) if is_known_plugin_event(name) => {
            KnownPluginEvent::deserialize(value).map(PluginEvent::from)
        }
        Some(_) => Ok(PluginEvent::Unknown(value)),
        None => KnownPluginEvent::deserialize(value).map(PluginEvent::from),
    }
}

fn is_known_plugin_event(name: &str) -> bool {
    matches!(
        name,
        "applicationDidLaunch"
            | "applicationDidTerminate"
            | "deviceDidChange"
            | "deviceDidConnect"
            | "deviceDidDisconnect"
            | "dialDown"
            | "dialUp"
            | "dialRotate"
            | "didReceiveDeepLink"
            | "didReceiveGlobalSettings"
            | "didReceiveSecrets"
            | "didReceiveSettings"
            | "didReceiveResources"
            | "keyDown"
            | "keyUp"
            | "propertyInspectorDidAppear"
            | "propertyInspectorDidDisappear"
            | "sendToPlugin"
            | "systemDidWakeUp"
            | "titleParametersDidChange"
            | "touchTap"
            | "willAppear"
            | "willDisappear"
    )
}

impl PluginEvent {
    pub fn name(&self) -> &str {
        match self {
            Self::ApplicationDidLaunch { .. } => "applicationDidLaunch",
            Self::ApplicationDidTerminate { .. } => "applicationDidTerminate",
            Self::DeviceDidChange { .. } => "deviceDidChange",
            Self::DeviceDidConnect { .. } => "deviceDidConnect",
            Self::DeviceDidDisconnect { .. } => "deviceDidDisconnect",
            Self::DialDown(_) => "dialDown",
            Self::DialUp(_) => "dialUp",
            Self::DialRotate(_) => "dialRotate",
            Self::DidReceiveDeepLink { .. } => "didReceiveDeepLink",
            Self::DidReceiveGlobalSettings { .. } => "didReceiveGlobalSettings",
            Self::DidReceiveSecrets { .. } => "didReceiveSecrets",
            Self::DidReceiveSettings(_) => "didReceiveSettings",
            Self::DidReceiveResources(_) => "didReceiveResources",
            Self::KeyDown(_) => "keyDown",
            Self::KeyUp(_) => "keyUp",
            Self::PropertyInspectorDidAppear(_) => "propertyInspectorDidAppear",
            Self::PropertyInspectorDidDisappear(_) => "propertyInspectorDidDisappear",
            Self::SendToPlugin { .. } => "sendToPlugin",
            Self::SystemDidWakeUp => "systemDidWakeUp",
            Self::TitleParametersDidChange(_) => "titleParametersDidChange",
            Self::TouchTap(_) => "touchTap",
            Self::WillAppear(_) => "willAppear",
            Self::WillDisappear(_) => "willDisappear",
            Self::Unknown(value) => value
                .get("event")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
        }
    }

    pub fn response_id(&self) -> Option<&str> {
        match self {
            Self::DidReceiveGlobalSettings { id, .. } | Self::DidReceiveSecrets { id, .. } => {
                id.as_deref()
            }
            Self::DidReceiveSettings(m) | Self::DidReceiveResources(m) => m.id.as_deref(),
            _ => None,
        }
    }

    pub fn context(&self) -> Option<&str> {
        match self {
            Self::DialDown(m) | Self::DialUp(m) => Some(m.context.as_str()),
            Self::DialRotate(m) => Some(m.context.as_str()),
            Self::DidReceiveSettings(m) | Self::DidReceiveResources(m) => Some(m.context.as_str()),
            Self::KeyDown(m) | Self::KeyUp(m) => Some(m.context.as_str()),
            Self::PropertyInspectorDidAppear(m) | Self::PropertyInspectorDidDisappear(m) => {
                Some(m.context.as_str())
            }
            Self::SendToPlugin { context, .. } => Some(context.as_str()),
            Self::TitleParametersDidChange(m) => Some(m.context.as_str()),
            Self::TouchTap(m) => Some(m.context.as_str()),
            Self::WillAppear(m) | Self::WillDisappear(m) => Some(m.context.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionIdentifier {
    pub action: String,
    pub context: String,
    pub device: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionMessage<P> {
    pub action: String,
    pub context: String,
    pub device: String,
    pub payload: P,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionMessageWithId<P> {
    pub action: String,
    pub context: String,
    pub device: String,
    #[serde(default)]
    pub id: Option<String>,
    pub payload: P,
}

impl<P> ActionMessageWithId<P> {
    pub fn as_message(&self) -> ActionMessage<P>
    where
        P: Clone,
    {
        ActionMessage {
            action: self.action.clone(),
            context: self.context.clone(),
            device: self.device.clone(),
            payload: self.payload.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationPayload {
    pub application: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeepLinkPayload {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsOnlyPayload {
    pub settings: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecretsPayload {
    pub secrets: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearPayload {
    #[serde(default)]
    pub settings: Value,
    pub controller: Controller,
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
    #[serde(default)]
    pub is_in_multi_action: bool,
    #[serde(default)]
    pub resources: Map<String, Value>,
    #[serde(default)]
    pub state: Option<State>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyGesturePayload {
    #[serde(default)]
    pub settings: Value,
    #[serde(default)]
    pub controller: Option<Controller>,
    #[serde(default)]
    pub coordinates: Option<Coordinates>,
    #[serde(default)]
    pub is_in_multi_action: bool,
    #[serde(default)]
    pub resources: Map<String, Value>,
    #[serde(default)]
    pub state: Option<State>,
    #[serde(default)]
    pub user_desired_state: Option<State>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderPayload {
    #[serde(default)]
    pub settings: Value,
    #[serde(default)]
    pub controller: Option<Controller>,
    pub coordinates: Coordinates,
    #[serde(default)]
    pub resources: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialRotatePayload {
    #[serde(flatten)]
    pub encoder: EncoderPayload,
    pub pressed: bool,
    pub ticks: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TouchTapPayload {
    #[serde(flatten)]
    pub encoder: EncoderPayload,
    pub hold: bool,
    pub tap_pos: [i32; 2],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParametersPayload {
    #[serde(default)]
    pub settings: Value,
    pub controller: Controller,
    pub coordinates: Coordinates,
    #[serde(default)]
    pub resources: Map<String, Value>,
    #[serde(default)]
    pub state: Option<State>,
    pub title: String,
    pub title_parameters: TitleParameters,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleParameters {
    pub font_family: String,
    pub font_size: u32,
    pub font_style: String,
    pub font_underline: bool,
    pub show_title: bool,
    pub title_alignment: String,
    pub title_color: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_will_appear() {
        let json = r#"{
            "event": "willAppear",
            "action": "com.elgato.test.one",
            "context": "context123",
            "device": "device123",
            "payload": {
                "controller": "Keypad",
                "coordinates": { "column": 8, "row": 2 },
                "isInMultiAction": false,
                "resources": {},
                "settings": { "name": "Elgato" },
                "state": 1
            }
        }"#;
        let ev: PluginEvent = serde_json::from_str(json).unwrap();
        match ev {
            PluginEvent::WillAppear(m) => {
                assert_eq!(m.action, "com.elgato.test.one");
                assert_eq!(m.payload.coordinates.unwrap().column, 8);
                assert!(!m.payload.is_in_multi_action);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn deserializes_system_wake() {
        let ev: PluginEvent = serde_json::from_str(r#"{"event":"systemDidWakeUp"}"#).unwrap();
        assert!(matches!(ev, PluginEvent::SystemDidWakeUp));
    }

    #[test]
    fn will_appear_keeps_unknown_controller() {
        let json = r#"{
            "event": "willAppear",
            "action": "com.elgato.test.one",
            "context": "context123",
            "device": "device123",
            "payload": {
                "controller": "Touchscreen",
                "coordinates": { "column": 0, "row": 0 },
                "isInMultiAction": false,
                "resources": {},
                "settings": {}
            }
        }"#;
        let ev: PluginEvent = serde_json::from_str(json).unwrap();
        match ev {
            PluginEvent::WillAppear(m) => {
                assert_eq!(
                    m.payload.controller,
                    Controller::Unknown("Touchscreen".into())
                );
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn keeps_unknown_event_json() {
        let json = r#"{"event":"didReceiveNewThing","payload":{"x":1},"extra":true}"#;
        let ev: PluginEvent = serde_json::from_str(json).unwrap();
        assert_eq!(ev.name(), "didReceiveNewThing");
        match ev {
            PluginEvent::Unknown(value) => {
                assert_eq!(value["event"], "didReceiveNewThing");
                assert_eq!(value["payload"]["x"], 1);
                assert_eq!(value["extra"], true);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn malformed_known_event_still_fails() {
        let err = serde_json::from_str::<PluginEvent>(r#"{"event":"willAppear"}"#).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing field") || msg.contains("willAppear"),
            "{msg}"
        );
    }

    #[test]
    fn missing_event_field_still_fails() {
        assert!(serde_json::from_str::<PluginEvent>(r#"{"foo":1}"#).is_err());
    }
}
