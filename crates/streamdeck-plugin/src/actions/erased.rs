use futures_util::future::BoxFuture;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::events::*;

use super::singleton::SingletonAction;

pub(crate) trait ErasedAction: Send + Sync {
    fn uuid(&self) -> &'static str;
    fn on_will_appear(&self, ev: WillAppearEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_will_disappear(&self, ev: WillDisappearEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_key_down(&self, ev: KeyDownEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_key_up(&self, ev: KeyUpEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_dial_down(&self, ev: DialDownEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_dial_up(&self, ev: DialUpEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_dial_rotate(&self, ev: DialRotateEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_touch_tap(&self, ev: TouchTapEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_title_parameters_did_change(
        &self,
        ev: TitleParametersDidChangeEvent<Value>,
    ) -> BoxFuture<'_, ()>;
    fn on_did_receive_settings(&self, ev: DidReceiveSettingsEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_did_receive_resources(&self, ev: DidReceiveResourcesEvent<Value>) -> BoxFuture<'_, ()>;
    fn on_property_inspector_did_appear(
        &self,
        ev: PropertyInspectorDidAppearEvent,
    ) -> BoxFuture<'_, ()>;
    fn on_property_inspector_did_disappear(
        &self,
        ev: PropertyInspectorDidDisappearEvent,
    ) -> BoxFuture<'_, ()>;
    fn on_send_to_plugin(&self, ev: SendToPluginEvent<Value>) -> BoxFuture<'_, ()>;
}

/// Stored settings can predate the current `Settings` type (for example after a
/// settings-structure change without a migration); the handler is skipped, but the
/// failure must be visible in the plugin log rather than a silently dead action.
fn warn_decode_failure(uuid: &str, event: &str, err: &serde_json::Error) {
    if let Ok(runtime) = crate::runtime::try_runtime() {
        runtime.logger.warn(format!(
            "Skipping {event} for {uuid}; failed to deserialize settings: {err}"
        ));
    }
}

impl<A: SingletonAction> ErasedAction for A {
    fn uuid(&self) -> &'static str {
        A::UUID
    }

    fn on_will_appear(&self, ev: WillAppearEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_will_appear::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_will_appear(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "willAppear", &err),
            }
        })
    }

    fn on_will_disappear(&self, ev: WillDisappearEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_will_disappear::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_will_disappear(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "willDisappear", &err),
            }
        })
    }

    fn on_key_down(&self, ev: KeyDownEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_key::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_key_down(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "keyDown", &err),
            }
        })
    }

    fn on_key_up(&self, ev: KeyUpEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_key::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_key_up(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "keyUp", &err),
            }
        })
    }

    fn on_dial_down(&self, ev: DialDownEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_dial::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_dial_down(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "dialDown", &err),
            }
        })
    }

    fn on_dial_up(&self, ev: DialUpEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_dial::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_dial_up(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "dialUp", &err),
            }
        })
    }

    fn on_dial_rotate(&self, ev: DialRotateEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_dial_rotate::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_dial_rotate(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "dialRotate", &err),
            }
        })
    }

    fn on_touch_tap(&self, ev: TouchTapEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_touch::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_touch_tap(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "touchTap", &err),
            }
        })
    }

    fn on_title_parameters_did_change(
        &self,
        ev: TitleParametersDidChangeEvent<Value>,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_title::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_title_parameters_did_change(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "titleParametersDidChange", &err),
            }
        })
    }

    fn on_did_receive_settings(&self, ev: DidReceiveSettingsEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_settings::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_did_receive_settings(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "didReceiveSettings", &err),
            }
        })
    }

    fn on_did_receive_resources(&self, ev: DidReceiveResourcesEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            match map_resources::<A::Settings>(ev) {
                Ok(ev) => SingletonAction::on_did_receive_resources(self, ev).await,
                Err(err) => warn_decode_failure(A::UUID, "didReceiveResources", &err),
            }
        })
    }

    fn on_property_inspector_did_appear(
        &self,
        ev: PropertyInspectorDidAppearEvent,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            SingletonAction::on_property_inspector_did_appear(self, ev).await;
        })
    }

    fn on_property_inspector_did_disappear(
        &self,
        ev: PropertyInspectorDidDisappearEvent,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            SingletonAction::on_property_inspector_did_disappear(self, ev).await;
        })
    }

    fn on_send_to_plugin(&self, ev: SendToPluginEvent<Value>) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            SingletonAction::on_send_to_plugin(self, ev).await;
        })
    }
}

fn decode<S: DeserializeOwned>(value: &Value) -> Result<S, serde_json::Error> {
    serde_json::from_value(value.clone())
}

fn map_appear<S: DeserializeOwned>(
    p: AppearPayloadTyped<Value>,
) -> Result<AppearPayloadTyped<S>, serde_json::Error> {
    Ok(AppearPayloadTyped {
        settings: decode(&p.settings)?,
        controller: p.controller,
        coordinates: p.coordinates,
        is_in_multi_action: p.is_in_multi_action,
        resources: p.resources,
        state: p.state,
    })
}

fn map_will_appear<S: DeserializeOwned>(
    ev: WillAppearEvent<Value>,
) -> Result<WillAppearEvent<S>, serde_json::Error> {
    Ok(WillAppearEvent {
        action: ev.action,
        payload: map_appear(ev.payload)?,
    })
}

fn map_will_disappear<S: DeserializeOwned>(
    ev: WillDisappearEvent<Value>,
) -> Result<WillDisappearEvent<S>, serde_json::Error> {
    Ok(WillDisappearEvent {
        action: ev.action,
        payload: map_appear(ev.payload)?,
    })
}

fn map_key<S: DeserializeOwned>(
    ev: KeyDownEvent<Value>,
) -> Result<KeyDownEvent<S>, serde_json::Error> {
    Ok(KeyDownEvent {
        action: ev.action,
        payload: KeyPayloadTyped {
            settings: decode(&ev.payload.settings)?,
            controller: ev.payload.controller,
            coordinates: ev.payload.coordinates,
            is_in_multi_action: ev.payload.is_in_multi_action,
            resources: ev.payload.resources,
            state: ev.payload.state,
            user_desired_state: ev.payload.user_desired_state,
        },
    })
}

fn map_dial<S: DeserializeOwned>(
    ev: DialDownEvent<Value>,
) -> Result<DialDownEvent<S>, serde_json::Error> {
    Ok(DialDownEvent {
        action: ev.action,
        payload: EncoderPayloadTyped {
            settings: decode(&ev.payload.settings)?,
            coordinates: ev.payload.coordinates,
            resources: ev.payload.resources,
        },
    })
}

fn map_dial_rotate<S: DeserializeOwned>(
    ev: DialRotateEvent<Value>,
) -> Result<DialRotateEvent<S>, serde_json::Error> {
    Ok(DialRotateEvent {
        action: ev.action,
        payload: DialRotatePayloadTyped {
            encoder: EncoderPayloadTyped {
                settings: decode(&ev.payload.encoder.settings)?,
                coordinates: ev.payload.encoder.coordinates,
                resources: ev.payload.encoder.resources,
            },
            pressed: ev.payload.pressed,
            ticks: ev.payload.ticks,
        },
    })
}

fn map_touch<S: DeserializeOwned>(
    ev: TouchTapEvent<Value>,
) -> Result<TouchTapEvent<S>, serde_json::Error> {
    Ok(TouchTapEvent {
        action: ev.action,
        payload: TouchTapPayloadTyped {
            encoder: EncoderPayloadTyped {
                settings: decode(&ev.payload.encoder.settings)?,
                coordinates: ev.payload.encoder.coordinates,
                resources: ev.payload.encoder.resources,
            },
            hold: ev.payload.hold,
            tap_pos: ev.payload.tap_pos,
        },
    })
}

fn map_title<S: DeserializeOwned>(
    ev: TitleParametersDidChangeEvent<Value>,
) -> Result<TitleParametersDidChangeEvent<S>, serde_json::Error> {
    Ok(TitleParametersDidChangeEvent {
        action: ev.action,
        payload: TitlePayloadTyped {
            settings: decode(&ev.payload.settings)?,
            controller: ev.payload.controller,
            coordinates: ev.payload.coordinates,
            resources: ev.payload.resources,
            state: ev.payload.state,
            title: ev.payload.title,
            title_parameters: ev.payload.title_parameters,
        },
    })
}

fn map_settings<S: DeserializeOwned>(
    ev: DidReceiveSettingsEvent<Value>,
) -> Result<DidReceiveSettingsEvent<S>, serde_json::Error> {
    Ok(DidReceiveSettingsEvent {
        action: ev.action,
        payload: map_appear(ev.payload)?,
    })
}

fn map_resources<S: DeserializeOwned>(
    ev: DidReceiveResourcesEvent<Value>,
) -> Result<DidReceiveResourcesEvent<S>, serde_json::Error> {
    Ok(DidReceiveResourcesEvent {
        action: ev.action,
        payload: map_appear(ev.payload)?,
    })
}
