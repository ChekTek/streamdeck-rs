use serde_json::Value;

use crate::actions::{Action, ActionContext, DialAction, KeyAction};
use crate::devices::Device;
use crate::protocol::{AppearPayload, Controller, Coordinates, Resources, State, TitleParameters};

#[derive(Clone, Debug)]
pub struct AppearPayloadTyped<S> {
    pub settings: S,
    pub controller: Controller,
    pub coordinates: Option<Coordinates>,
    pub is_in_multi_action: bool,
    pub resources: Resources,
    pub state: Option<State>,
}

impl AppearPayloadTyped<Value> {
    pub fn from_protocol(p: &AppearPayload) -> Self {
        Self {
            settings: p.settings.clone(),
            controller: p.controller.clone(),
            coordinates: p.coordinates,
            is_in_multi_action: p.is_in_multi_action,
            resources: p.resources.clone(),
            state: p.state,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyPayloadTyped<S> {
    pub settings: S,
    pub controller: Option<Controller>,
    pub coordinates: Option<Coordinates>,
    pub is_in_multi_action: bool,
    pub resources: Resources,
    pub state: Option<State>,
    pub user_desired_state: Option<State>,
}

#[derive(Clone, Debug)]
pub struct EncoderPayloadTyped<S> {
    pub settings: S,
    pub coordinates: Coordinates,
    pub resources: Resources,
}

#[derive(Clone, Debug)]
pub struct DialRotatePayloadTyped<S> {
    pub encoder: EncoderPayloadTyped<S>,
    pub pressed: bool,
    pub ticks: i32,
}

#[derive(Clone, Debug)]
pub struct TouchTapPayloadTyped<S> {
    pub encoder: EncoderPayloadTyped<S>,
    pub hold: bool,
    pub tap_pos: [i32; 2],
}

#[derive(Clone, Debug)]
pub struct TitlePayloadTyped<S> {
    pub settings: S,
    pub controller: Controller,
    pub coordinates: Coordinates,
    pub resources: Resources,
    pub state: Option<State>,
    pub title: String,
    pub title_parameters: TitleParameters,
}

#[derive(Clone)]
pub struct WillAppearEvent<S> {
    pub action: Action,
    pub payload: AppearPayloadTyped<S>,
}

#[derive(Clone)]
pub struct WillDisappearEvent<S> {
    pub action: ActionContext,
    pub payload: AppearPayloadTyped<S>,
}

#[derive(Clone)]
pub struct KeyDownEvent<S> {
    pub action: KeyAction,
    pub payload: KeyPayloadTyped<S>,
}

pub type KeyUpEvent<S> = KeyDownEvent<S>;

#[derive(Clone)]
pub struct DialDownEvent<S> {
    pub action: DialAction,
    pub payload: EncoderPayloadTyped<S>,
}

pub type DialUpEvent<S> = DialDownEvent<S>;

#[derive(Clone)]
pub struct DialRotateEvent<S> {
    pub action: DialAction,
    pub payload: DialRotatePayloadTyped<S>,
}

#[derive(Clone)]
pub struct TouchTapEvent<S> {
    pub action: DialAction,
    pub payload: TouchTapPayloadTyped<S>,
}

#[derive(Clone)]
pub struct TitleParametersDidChangeEvent<S> {
    pub action: Action,
    pub payload: TitlePayloadTyped<S>,
}

#[derive(Clone)]
pub struct DidReceiveSettingsEvent<S> {
    pub action: Action,
    pub payload: AppearPayloadTyped<S>,
}

#[derive(Clone)]
pub struct DidReceiveResourcesEvent<S> {
    pub action: Action,
    pub payload: AppearPayloadTyped<S>,
}

#[derive(Clone)]
pub struct PropertyInspectorDidAppearEvent {
    pub action: Action,
}

pub type PropertyInspectorDidDisappearEvent = PropertyInspectorDidAppearEvent;

#[derive(Clone)]
pub struct SendToPluginEvent<P> {
    pub action: Action,
    pub payload: P,
}

#[derive(Clone)]
pub struct DidReceiveGlobalSettingsEvent<S> {
    pub settings: S,
}

#[derive(Clone)]
pub struct ApplicationDidLaunchEvent {
    pub application: String,
}

pub type ApplicationDidTerminateEvent = ApplicationDidLaunchEvent;

#[derive(Clone)]
pub struct SystemDidWakeUpEvent;

#[derive(Clone)]
pub struct DeviceDidConnectEvent {
    pub device: Device,
}

pub type DeviceDidDisconnectEvent = DeviceDidConnectEvent;
pub type DeviceDidChangeEvent = DeviceDidConnectEvent;

#[derive(Clone)]
pub struct DidReceiveDeepLinkEvent {
    pub url: DeepLinkUrl,
}

/// Deep-link URL routed from Stream Deck (schema/authority omitted).
#[derive(Clone, Debug)]
pub struct DeepLinkUrl {
    pub href: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
    pub query_parameters: Vec<(String, String)>,
}

impl DeepLinkUrl {
    pub fn parse(url: &str) -> Self {
        let href = url.to_string();
        let (without_frag, fragment) = match href.split_once('#') {
            Some((a, b)) => (a, b.to_string()),
            None => (href.as_str(), String::new()),
        };
        let (path, query) = match without_frag.split_once('?') {
            Some((a, b)) => (a.to_string(), b.to_string()),
            None => (without_frag.to_string(), String::new()),
        };
        let query_parameters = query
            .split('&')
            .filter(|p| !p.is_empty())
            .map(|pair| match pair.split_once('=') {
                Some((k, v)) => (percent_decode(k), percent_decode(v)),
                None => (percent_decode(pair), String::new()),
            })
            .collect();
        Self {
            href,
            path,
            query,
            fragment,
            query_parameters,
        }
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2]))
        {
            out.push((hi << 4) | lo);
            i += 3;
            continue;
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_deep_link() {
        let url = DeepLinkUrl::parse("/test?name=elgato&key=123#heading");
        assert_eq!(url.path, "/test");
        assert_eq!(url.query, "name=elgato&key=123");
        assert_eq!(url.fragment, "heading");
        assert_eq!(url.query_parameters[0], ("name".into(), "elgato".into()));
    }

    #[test]
    fn percent_decode_skips_multibyte_after_percent() {
        let url = DeepLinkUrl::parse("/test?q=%€&ok=%2F");
        assert_eq!(url.query_parameters[0], ("q".into(), "%€".into()));
        assert_eq!(url.query_parameters[1], ("ok".into(), "/".into()));
    }
}
