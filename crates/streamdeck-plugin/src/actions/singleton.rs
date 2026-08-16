use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::events::{
    DialDownEvent, DialRotateEvent, DialUpEvent, DidReceiveResourcesEvent, DidReceiveSettingsEvent,
    KeyDownEvent, KeyUpEvent, PropertyInspectorDidAppearEvent, PropertyInspectorDidDisappearEvent,
    SendToPluginEvent, TitleParametersDidChangeEvent, TouchTapEvent, WillAppearEvent,
    WillDisappearEvent,
};

use super::Action;

/// Per-UUID action handler. Optional methods default to no-ops.
///
/// One instance receives events for every visible instance of that action UUID.
///
/// Panics in these callbacks are caught, logged, and do not stop the plugin.
/// Heavy CPU work (image rasterization, font shaping) should run on
/// [`tokio::task::spawn_blocking`] rather than on the Tokio worker that
/// drives `willAppear` / `keyDown`.
pub trait SingletonAction: Send + Sync + 'static {
    /// Manifest action UUID (`Actions[].UUID`).
    const UUID: &'static str;

    /// Settings object stored on each action instance.
    type Settings: DeserializeOwned + Serialize + Send + Sync + Clone + 'static;

    fn on_dial_down(&self, ev: DialDownEvent<Self::Settings>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_dial_rotate(
        &self,
        ev: DialRotateEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_dial_up(&self, ev: DialUpEvent<Self::Settings>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_did_receive_resources(
        &self,
        ev: DidReceiveResourcesEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_did_receive_settings(
        &self,
        ev: DidReceiveSettingsEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_key_up(&self, ev: KeyUpEvent<Self::Settings>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_property_inspector_did_appear(
        &self,
        ev: PropertyInspectorDidAppearEvent,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_property_inspector_did_disappear(
        &self,
        ev: PropertyInspectorDidDisappearEvent,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_send_to_plugin(&self, ev: SendToPluginEvent<Value>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_title_parameters_did_change(
        &self,
        ev: TitleParametersDidChangeEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_touch_tap(&self, ev: TouchTapEvent<Self::Settings>) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_will_appear(
        &self,
        ev: WillAppearEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    fn on_will_disappear(
        &self,
        ev: WillDisappearEvent<Self::Settings>,
    ) -> impl Future<Output = ()> + Send {
        async {
            let _ = ev;
        }
    }

    /// Visible instances of this action UUID.
    fn actions<'a>(&self, all: &'a [Action]) -> Vec<&'a Action> {
        all.iter()
            .filter(|a| a.manifest_id() == Self::UUID)
            .collect()
    }
}
