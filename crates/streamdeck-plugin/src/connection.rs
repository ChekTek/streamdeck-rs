use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::actions::{Action, ActionContext, ActionHandle, DialAction, KeyAction};
use crate::devices::Device;
use crate::error::{Error, Result};
use crate::events::*;
use crate::protocol::{
    ActionMessage, AppearPayload, Controller, EncoderPayload, KeyGesturePayload, PluginEvent,
    RegisterEvent,
};
use crate::runtime::Runtime;

/// Connect, register, and run until the WebSocket closes.
pub async fn run(runtime: Arc<Runtime>) -> Result<()> {
    let reader = start(runtime).await?;
    reader
        .await
        .map_err(|e| Error::Message(format!("connection task: {e}")))?
}

/// Establish the connection and spawn the read loop. Returns a join handle.
pub async fn start(runtime: Arc<Runtime>) -> Result<tokio::task::JoinHandle<Result<()>>> {
    let url = runtime.registration.websocket_url();
    let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
    let mut ws = ws;

    let register = RegisterEvent {
        event: runtime.registration.register_event.clone(),
        uuid: runtime.registration.plugin_uuid.clone(),
    };
    let register_json = serde_json::to_string(&register)?;
    runtime
        .logger
        .create_scope("Connection")
        .trace(&register_json);
    ws.send(Message::Text(register_json.into())).await?;

    runtime.seed_devices();

    let (mut write, mut read) = ws.split();
    let mut outgoing = runtime
        .take_outgoing_rx()
        .ok_or_else(|| Error::Message("connection already started".into()))?;

    let write_runtime = runtime.clone();
    tokio::spawn(async move {
        while let Some(text) = outgoing.recv().await {
            if write.send(Message::Text(text.into())).await.is_err() {
                write_runtime
                    .logger
                    .create_scope("Connection")
                    .warn("failed to send message");
                break;
            }
        }
    });

    // Broadcast inbound events immediately so request/response APIs can complete
    // while action callbacks run on a separate ordered worker.
    let (dispatch_tx, mut dispatch_rx) = mpsc::unbounded_channel();
    let worker_runtime = runtime.clone();
    let worker = tokio::spawn(async move {
        while let Some(event) = dispatch_rx.recv().await {
            dispatch(&worker_runtime, event).await;
        }
    });

    let reader_runtime = runtime.clone();
    Ok(tokio::spawn(async move {
        let mut read_result = Ok(());
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    handle_text(&reader_runtime, text.as_str(), &dispatch_tx);
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) | Ok(Message::Binary(_)) => {}
                Ok(Message::Close(_)) | Ok(Message::Frame(_)) => break,
                Err(err) => {
                    reader_runtime
                        .logger
                        .create_scope("Connection")
                        .error(format!("WebSocket error: {err}"));
                    read_result = Err(err.into());
                    break;
                }
            }
        }
        drop(dispatch_tx);
        match worker.await {
            Ok(()) => read_result,
            Err(err) if read_result.is_ok() => Err(Error::Message(format!("dispatch task: {err}"))),
            Err(_) => read_result,
        }
    }))
}

fn handle_text(
    runtime: &Arc<Runtime>,
    text: &str,
    dispatch_tx: &mpsc::UnboundedSender<PluginEvent>,
) {
    let logged = redact_inbound(text);
    runtime.logger.create_scope("Connection").trace(&logged);
    match serde_json::from_str::<PluginEvent>(text) {
        Ok(event) => {
            let _ = runtime.events.send(Arc::new(event.clone()));
            if dispatch_tx.send(event).is_err() {
                runtime
                    .logger
                    .create_scope("Connection")
                    .warn("dispatch worker stopped");
            }
        }
        Err(err) => {
            if let Ok(value) = serde_json::from_str::<Value>(text) {
                if value.get("event").and_then(|e| e.as_str()).is_none() {
                    runtime
                        .logger
                        .create_scope("Connection")
                        .warn(format!("Received unknown message: {logged}"));
                } else {
                    runtime
                        .logger
                        .create_scope("Connection")
                        .error(format!("Failed to parse message: {logged} ({err})"));
                }
            } else {
                runtime
                    .logger
                    .create_scope("Connection")
                    .error(format!("Failed to parse message: {logged} ({err})"));
            }
        }
    }
}

fn redact_inbound(text: &str) -> String {
    match serde_json::from_str::<Value>(text) {
        Ok(mut value) => {
            if value.get("event").and_then(Value::as_str) == Some("didReceiveSecrets") {
                if let Some(payload) = value.get_mut("payload").and_then(Value::as_object_mut) {
                    payload.insert("secrets".into(), Value::String("[redacted]".into()));
                }
                value.to_string()
            } else {
                text.to_string()
            }
        }
        Err(_) if text.contains("didReceiveSecrets") => "[redacted didReceiveSecrets]".into(),
        Err(_) => text.to_string(),
    }
}

async fn dispatch(runtime: &Arc<Runtime>, event: PluginEvent) {
    match event {
        PluginEvent::WillAppear(msg) => on_will_appear(runtime, msg).await,
        PluginEvent::WillDisappear(msg) => on_will_disappear(runtime, msg).await,
        PluginEvent::KeyDown(msg) => on_key(runtime, msg, true).await,
        PluginEvent::KeyUp(msg) => on_key(runtime, msg, false).await,
        PluginEvent::DialDown(msg) => on_dial(runtime, msg, DialKind::Down).await,
        PluginEvent::DialUp(msg) => on_dial(runtime, msg, DialKind::Up).await,
        PluginEvent::DialRotate(msg) => on_dial_rotate(runtime, msg).await,
        PluginEvent::TouchTap(msg) => on_touch_tap(runtime, msg).await,
        PluginEvent::TitleParametersDidChange(msg) => on_title(runtime, msg).await,
        PluginEvent::DidReceiveSettings(msg) => on_did_receive_settings(runtime, msg).await,
        PluginEvent::DidReceiveResources(msg) => on_did_receive_resources(runtime, msg).await,
        PluginEvent::DidReceiveGlobalSettings { id, payload } => {
            on_global_settings(runtime, id, payload.settings).await;
        }
        PluginEvent::PropertyInspectorDidAppear(id) => {
            on_pi(runtime, &id.context, true).await;
        }
        PluginEvent::PropertyInspectorDidDisappear(id) => {
            on_pi(runtime, &id.context, false).await;
        }
        PluginEvent::SendToPlugin {
            action: _,
            context,
            payload,
        } => on_send_to_plugin(runtime, &context, payload).await,
        PluginEvent::DeviceDidConnect {
            device,
            device_info,
        } => {
            let dev = if let Some(existing) = runtime.device_store.get(&device) {
                existing.set_info(device_info);
                existing.set_connected(true);
                existing
            } else {
                let d = Device::new(device, device_info, true);
                runtime.device_store.set(d.clone());
                d
            };
            runtime
                .listeners
                .device_did_connect
                .emit(DeviceDidConnectEvent { device: dev })
                .await;
        }
        PluginEvent::DeviceDidDisconnect { device } => {
            if let Some(dev) = runtime.device_store.get(&device) {
                dev.set_connected(false);
                runtime
                    .listeners
                    .device_did_disconnect
                    .emit(DeviceDidDisconnectEvent { device: dev })
                    .await;
            }
        }
        PluginEvent::DeviceDidChange {
            device,
            device_info,
        } => {
            let dev = if let Some(existing) = runtime.device_store.get(&device) {
                existing.set_info(device_info);
                existing
            } else {
                let d = Device::new(device, device_info, false);
                runtime.device_store.set(d.clone());
                d
            };
            runtime
                .listeners
                .device_did_change
                .emit(DeviceDidChangeEvent { device: dev })
                .await;
        }
        PluginEvent::ApplicationDidLaunch { payload } => {
            runtime
                .listeners
                .application_did_launch
                .emit(ApplicationDidLaunchEvent {
                    application: payload.application,
                })
                .await;
        }
        PluginEvent::ApplicationDidTerminate { payload } => {
            runtime
                .listeners
                .application_did_terminate
                .emit(ApplicationDidTerminateEvent {
                    application: payload.application,
                })
                .await;
        }
        PluginEvent::DidReceiveDeepLink { payload } => {
            runtime
                .listeners
                .did_receive_deep_link
                .emit(DidReceiveDeepLinkEvent {
                    url: DeepLinkUrl::parse(&payload.url),
                })
                .await;
        }
        PluginEvent::SystemDidWakeUp => {
            runtime
                .listeners
                .system_did_wake_up
                .emit(SystemDidWakeUpEvent)
                .await;
        }
        PluginEvent::DidReceiveSecrets { .. } => {}
    }
}

fn make_handle(
    runtime: &Arc<Runtime>,
    action: &str,
    context: &str,
    device: &str,
    controller: Controller,
) -> Result<ActionHandle> {
    let device = runtime
        .device_store
        .get(device)
        .ok_or_else(|| Error::DeviceNotFound(device.to_string()))?;
    Ok(ActionHandle {
        runtime: runtime.clone(),
        id: context.to_string(),
        manifest_id: action.to_string(),
        device,
        controller,
    })
}

fn action_from_appear(
    runtime: &Arc<Runtime>,
    msg: &ActionMessage<AppearPayload>,
) -> Result<Action> {
    let handle = make_handle(
        runtime,
        &msg.action,
        &msg.context,
        &msg.device,
        msg.payload.controller,
    )?;
    match msg.payload.controller {
        Controller::Keypad => {
            let coords = if msg.payload.is_in_multi_action {
                None
            } else {
                msg.payload.coordinates
            };
            Ok(Action::Key(KeyAction::new(
                handle,
                coords,
                msg.payload.is_in_multi_action,
            )))
        }
        Controller::Encoder => {
            let coords = msg
                .payload
                .coordinates
                .ok_or_else(|| Error::Message("encoder willAppear missing coordinates".into()))?;
            Ok(Action::Dial(DialAction::new(handle, coords)))
        }
    }
}

async fn on_will_appear(runtime: &Arc<Runtime>, msg: ActionMessage<AppearPayload>) {
    let Ok(action) = action_from_appear(runtime, &msg) else {
        runtime.logger.error(format!(
            "Failed to initialize action; device {} not found",
            msg.device
        ));
        return;
    };
    if runtime.experimental_ids() {
        runtime
            .settings_cache
            .lock()
            .expect("cache")
            .insert(msg.context.clone(), msg.payload.settings.clone());
    }
    runtime.action_store.set(action.clone());
    let ev = WillAppearEvent {
        action: action.clone(),
        payload: AppearPayloadTyped::from_protocol(&msg.payload),
    };
    for a in registered(runtime, action.manifest_id()) {
        a.on_will_appear(ev.clone()).await;
    }
    runtime.listeners.will_appear.emit(ev).await;
}

async fn on_will_disappear(runtime: &Arc<Runtime>, msg: ActionMessage<AppearPayload>) {
    let device = match runtime.device_store.get(&msg.device) {
        Some(d) => d,
        None => {
            runtime.logger.error(format!(
                "Failed to initialize action; device {} not found",
                msg.device
            ));
            return;
        }
    };
    let ctx = ActionContext {
        id: msg.context.clone(),
        manifest_id: msg.action.clone(),
        device,
        controller: msg.payload.controller,
    };
    runtime.action_store.delete(&msg.context);
    runtime
        .settings_cache
        .lock()
        .expect("cache")
        .remove(&msg.context);
    let ev = WillDisappearEvent {
        action: ctx,
        payload: AppearPayloadTyped::from_protocol(&msg.payload),
    };
    for a in registered(runtime, &msg.action) {
        a.on_will_disappear(ev.clone()).await;
    }
    runtime.listeners.will_disappear.emit(ev).await;
}

async fn on_key(runtime: &Arc<Runtime>, msg: ActionMessage<KeyGesturePayload>, down: bool) {
    let Some(Action::Key(key)) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = KeyDownEvent {
        action: key,
        payload: KeyPayloadTyped {
            settings: msg.payload.settings,
            controller: msg.payload.controller,
            coordinates: msg.payload.coordinates,
            is_in_multi_action: msg.payload.is_in_multi_action,
            resources: msg.payload.resources,
            state: msg.payload.state,
            user_desired_state: msg.payload.user_desired_state,
        },
    };
    let uuid = ev.action.manifest_id().to_string();
    if down {
        for a in registered(runtime, &uuid) {
            a.on_key_down(ev.clone()).await;
        }
        runtime.listeners.key_down.emit(ev).await;
    } else {
        for a in registered(runtime, &uuid) {
            a.on_key_up(ev.clone()).await;
        }
        runtime.listeners.key_up.emit(ev).await;
    }
}

enum DialKind {
    Down,
    Up,
}

async fn on_dial(runtime: &Arc<Runtime>, msg: ActionMessage<EncoderPayload>, kind: DialKind) {
    let Some(Action::Dial(dial)) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = DialDownEvent {
        action: dial,
        payload: encoder_typed(&msg.payload),
    };
    let uuid = ev.action.manifest_id().to_string();
    match kind {
        DialKind::Down => {
            for a in registered(runtime, &uuid) {
                a.on_dial_down(ev.clone()).await;
            }
            runtime.listeners.dial_down.emit(ev).await;
        }
        DialKind::Up => {
            for a in registered(runtime, &uuid) {
                a.on_dial_up(ev.clone()).await;
            }
            runtime.listeners.dial_up.emit(ev).await;
        }
    }
}

async fn on_dial_rotate(
    runtime: &Arc<Runtime>,
    msg: ActionMessage<crate::protocol::DialRotatePayload>,
) {
    let Some(Action::Dial(dial)) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = DialRotateEvent {
        action: dial,
        payload: DialRotatePayloadTyped {
            encoder: encoder_typed(&msg.payload.encoder),
            pressed: msg.payload.pressed,
            ticks: msg.payload.ticks,
        },
    };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_dial_rotate(ev.clone()).await;
    }
    runtime.listeners.dial_rotate.emit(ev).await;
}

async fn on_touch_tap(
    runtime: &Arc<Runtime>,
    msg: ActionMessage<crate::protocol::TouchTapPayload>,
) {
    let Some(Action::Dial(dial)) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = TouchTapEvent {
        action: dial,
        payload: TouchTapPayloadTyped {
            encoder: encoder_typed(&msg.payload.encoder),
            hold: msg.payload.hold,
            tap_pos: msg.payload.tap_pos,
        },
    };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_touch_tap(ev.clone()).await;
    }
    runtime.listeners.touch_tap.emit(ev).await;
}

async fn on_title(
    runtime: &Arc<Runtime>,
    msg: ActionMessage<crate::protocol::TitleParametersPayload>,
) {
    let Some(action) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = TitleParametersDidChangeEvent {
        action,
        payload: TitlePayloadTyped {
            settings: msg.payload.settings,
            controller: msg.payload.controller,
            coordinates: msg.payload.coordinates,
            resources: msg.payload.resources,
            state: msg.payload.state,
            title: msg.payload.title,
            title_parameters: msg.payload.title_parameters,
        },
    };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_title_parameters_did_change(ev.clone()).await;
    }
    runtime.listeners.title_parameters.emit(ev).await;
}

async fn on_did_receive_settings(
    runtime: &Arc<Runtime>,
    msg: crate::protocol::ActionMessageWithId<AppearPayload>,
) {
    if runtime.experimental_ids() {
        runtime
            .settings_cache
            .lock()
            .expect("cache")
            .insert(msg.context.clone(), msg.payload.settings.clone());
        if msg.id.is_some() {
            return;
        }
    }
    let Some(action) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = DidReceiveSettingsEvent {
        action,
        payload: AppearPayloadTyped::from_protocol(&msg.payload),
    };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_did_receive_settings(ev.clone()).await;
    }
    runtime.listeners.did_receive_settings.emit(ev).await;
}

async fn on_did_receive_resources(
    runtime: &Arc<Runtime>,
    msg: crate::protocol::ActionMessageWithId<AppearPayload>,
) {
    if msg.id.is_some() {
        return;
    }
    let Some(action) = runtime.action_store.get(&msg.context) else {
        return;
    };
    let ev = DidReceiveResourcesEvent {
        action,
        payload: AppearPayloadTyped::from_protocol(&msg.payload),
    };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_did_receive_resources(ev.clone()).await;
    }
    runtime.listeners.did_receive_resources.emit(ev).await;
}

async fn on_global_settings(runtime: &Arc<Runtime>, id: Option<String>, settings: Value) {
    if runtime.experimental_ids() && id.is_some() {
        return;
    }
    runtime
        .listeners
        .did_receive_global_settings
        .emit(DidReceiveGlobalSettingsEvent { settings })
        .await;
}

async fn on_pi(runtime: &Arc<Runtime>, context: &str, appear: bool) {
    let Some(action) = runtime.action_store.get(context) else {
        return;
    };
    let id = action.id().to_string();
    let manifest = action.manifest_id().to_string();
    let device = action.device().id();
    if appear {
        if runtime.ui.is_current(&id, &manifest, &device) {
            runtime.ui.inc_stack();
        } else {
            runtime.ui.set_current(id, manifest, device);
        }
        let ev = PropertyInspectorDidAppearEvent { action };
        let uuid = ev.action.manifest_id().to_string();
        for a in registered(runtime, &uuid) {
            a.on_property_inspector_did_appear(ev.clone()).await;
        }
        runtime
            .listeners
            .property_inspector_did_appear
            .emit(ev)
            .await;
    } else {
        if runtime.ui.is_current(&id, &manifest, &device) {
            runtime.ui.dec_stack();
        }
        let ev = PropertyInspectorDidDisappearEvent { action };
        let uuid = ev.action.manifest_id().to_string();
        for a in registered(runtime, &uuid) {
            a.on_property_inspector_did_disappear(ev.clone()).await;
        }
        runtime
            .listeners
            .property_inspector_did_disappear
            .emit(ev)
            .await;
    }
}

async fn on_send_to_plugin(runtime: &Arc<Runtime>, context: &str, payload: Value) {
    let Some(action) = runtime.action_store.get(context) else {
        return;
    };
    let ev = SendToPluginEvent { action, payload };
    let uuid = ev.action.manifest_id().to_string();
    for a in registered(runtime, &uuid) {
        a.on_send_to_plugin(ev.clone()).await;
    }
    runtime.listeners.send_to_plugin.emit(ev).await;
}

fn encoder_typed(p: &EncoderPayload) -> EncoderPayloadTyped<Value> {
    EncoderPayloadTyped {
        settings: p.settings.clone(),
        coordinates: p.coordinates,
        resources: p.resources.clone(),
    }
}

fn registered(runtime: &Arc<Runtime>, uuid: &str) -> Vec<Arc<dyn crate::actions::ErasedAction>> {
    runtime
        .registered
        .read()
        .expect("registered")
        .iter()
        .filter(|a| a.uuid() == uuid)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;

    use crate::StreamDeck;
    use crate::actions::SingletonAction;

    struct HelloAction;

    impl SingletonAction for HelloAction {
        const UUID: &'static str = "com.elgato.test.one";
        type Settings = Value;

        async fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) {
            let _ = ev.action.set_title("Hello world").await;
        }
    }

    struct FetchSettingsAction;

    impl SingletonAction for FetchSettingsAction {
        const UUID: &'static str = "com.elgato.test.one";
        type Settings = Value;

        async fn on_key_down(&self, ev: KeyDownEvent<Self::Settings>) {
            let settings = ev
                .action
                .get_settings::<Value>()
                .await
                .expect("settings response");
            let title = settings
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("missing");
            let _ = ev.action.set_title(title).await;
        }
    }

    fn info_json() -> String {
        json!({
            "application": { "version": "7.1.0", "language": "en", "platform": "mac", "platformVersion": "14", "font": "Arial" },
            "plugin": { "uuid": "com.elgato.test", "version": "1.0" },
            "devices": [{ "id": "device123", "name": "Deck", "size": { "columns": 5, "rows": 3 }, "type": 0 }]
        })
        .to_string()
    }

    #[tokio::test]
    async fn registers_and_routes_key_down() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let info = info_json();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();
            let first = ws.next().await.unwrap().unwrap();
            let Message::Text(text) = first else {
                panic!("expected text")
            };
            let reg: Value = serde_json::from_str(text.as_str()).unwrap();
            assert_eq!(reg["event"], "registerPlugin");
            assert_eq!(reg["uuid"], "abc123");

            let will_appear = json!({
                "event": "willAppear",
                "action": "com.elgato.test.one",
                "context": "context123",
                "device": "device123",
                "payload": {
                    "controller": "Keypad",
                    "coordinates": { "column": 1, "row": 2 },
                    "isInMultiAction": false,
                    "resources": {},
                    "settings": {}
                }
            });
            ws.send(Message::Text(will_appear.to_string().into()))
                .await
                .unwrap();

            let key_down = json!({
                "event": "keyDown",
                "action": "com.elgato.test.one",
                "context": "context123",
                "device": "device123",
                "payload": {
                    "controller": "Keypad",
                    "coordinates": { "column": 1, "row": 2 },
                    "isInMultiAction": false,
                    "resources": {},
                    "settings": {}
                }
            });
            ws.send(Message::Text(key_down.to_string().into()))
                .await
                .unwrap();

            let reply = ws.next().await.unwrap().unwrap();
            let Message::Text(text) = reply else {
                panic!("expected text")
            };
            let cmd: Value = serde_json::from_str(text.as_str()).unwrap();
            let _ = ws.close(None).await;
            cmd
        });

        let plugin = tokio::spawn(async move {
            StreamDeck::from_args([
                "-port",
                &port.to_string(),
                "-pluginUUID",
                "abc123",
                "-registerEvent",
                "registerPlugin",
                "-info",
                &info,
            ])
            .unwrap()
            .register_action(HelloAction)
            .unwrap()
            .connect()
            .await
        });

        let cmd = tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .expect("server timed out")
            .unwrap();
        assert_eq!(cmd["event"], "setTitle");
        assert_eq!(cmd["context"], "context123");
        assert_eq!(cmd["payload"]["title"], "Hello world");
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), plugin).await;
    }

    #[tokio::test]
    async fn get_settings_from_callback_does_not_deadlock() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let info = info_json();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = accept_async(stream).await.unwrap();
            let first = ws.next().await.unwrap().unwrap();
            let Message::Text(_) = first else {
                panic!("expected text")
            };

            let will_appear = json!({
                "event": "willAppear",
                "action": "com.elgato.test.one",
                "context": "context123",
                "device": "device123",
                "payload": {
                    "controller": "Keypad",
                    "coordinates": { "column": 1, "row": 2 },
                    "isInMultiAction": false,
                    "resources": {},
                    "settings": {}
                }
            });
            ws.send(Message::Text(will_appear.to_string().into()))
                .await
                .unwrap();

            let key_down = json!({
                "event": "keyDown",
                "action": "com.elgato.test.one",
                "context": "context123",
                "device": "device123",
                "payload": {
                    "controller": "Keypad",
                    "coordinates": { "column": 1, "row": 2 },
                    "isInMultiAction": false,
                    "resources": {},
                    "settings": {}
                }
            });
            ws.send(Message::Text(key_down.to_string().into()))
                .await
                .unwrap();

            loop {
                let msg = ws.next().await.unwrap().unwrap();
                let Message::Text(text) = msg else {
                    panic!("expected text")
                };
                let cmd: Value = serde_json::from_str(text.as_str()).unwrap();
                match cmd["event"].as_str() {
                    Some("getSettings") => {
                        let reply = json!({
                            "event": "didReceiveSettings",
                            "action": "com.elgato.test.one",
                            "context": "context123",
                            "device": "device123",
                            "id": cmd.get("id").cloned(),
                            "payload": {
                                "controller": "Keypad",
                                "coordinates": { "column": 1, "row": 2 },
                                "isInMultiAction": false,
                                "resources": {},
                                "settings": { "label": "from-sd" }
                            }
                        });
                        ws.send(Message::Text(reply.to_string().into()))
                            .await
                            .unwrap();
                    }
                    Some("setTitle") => {
                        let _ = ws.close(None).await;
                        break cmd;
                    }
                    other => panic!("unexpected event {other:?}"),
                }
            }
        });

        let plugin = tokio::spawn(async move {
            StreamDeck::from_args([
                "-port",
                &port.to_string(),
                "-pluginUUID",
                "abc123",
                "-registerEvent",
                "registerPlugin",
                "-info",
                &info,
            ])
            .unwrap()
            .register_action(FetchSettingsAction)
            .unwrap()
            .connect()
            .await
        });

        let cmd = tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .expect("server timed out")
            .unwrap();
        assert_eq!(cmd["event"], "setTitle");
        assert_eq!(cmd["payload"]["title"], "from-sd");
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), plugin).await;
    }

    #[test]
    fn serializes_set_title() {
        let cmd = crate::protocol::PluginCommand::SetTitle {
            context: "c".into(),
            payload: crate::protocol::SetTitlePayload {
                title: Some("Hello world".into()),
                state: None,
                target: None,
            },
        };
        let v = serde_json::to_value(&cmd).unwrap();
        assert_eq!(v["event"], "setTitle");
        assert_eq!(v["context"], "c");
        assert_eq!(v["payload"]["title"], "Hello world");
        assert!(v["payload"].get("state").is_none());
    }

    #[test]
    fn redacts_secrets_payloads() {
        let raw = r#"{"event":"didReceiveSecrets","payload":{"secrets":{"token":"s3cret"}}}"#;
        let redacted = redact_inbound(raw);
        assert!(!redacted.contains("s3cret"));
        assert!(redacted.contains("didReceiveSecrets"));
        assert!(redacted.contains("[redacted]"));
    }
}
