use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde_json::Value;
use tokio::sync::{broadcast, mpsc};

use crate::actions::{ActionStore, ErasedAction};
use crate::devices::DeviceStore;
use crate::error::{Error, Result};
use crate::events::*;
use crate::listeners::ListenerSet;
use crate::logging::{Logger, plugin_uuid_from_cwd};
use crate::manifest::Manifest;
use crate::protocol::{PluginCommand, PluginEvent};
use crate::registration::RegistrationParameters;
use crate::version::Version;

static GLOBAL: Mutex<Option<Arc<Runtime>>> = Mutex::new(None);

pub(crate) fn set_runtime(runtime: Arc<Runtime>) {
    *GLOBAL.lock().expect("runtime") = Some(runtime);
}

pub fn try_runtime() -> Result<Arc<Runtime>> {
    GLOBAL
        .lock()
        .expect("runtime")
        .clone()
        .ok_or(Error::NotInitialized)
}

pub struct EventListeners {
    pub will_appear: Arc<ListenerSet<WillAppearEvent<Value>>>,
    pub will_disappear: Arc<ListenerSet<WillDisappearEvent<Value>>>,
    pub key_down: Arc<ListenerSet<KeyDownEvent<Value>>>,
    pub key_up: Arc<ListenerSet<KeyUpEvent<Value>>>,
    pub dial_down: Arc<ListenerSet<DialDownEvent<Value>>>,
    pub dial_up: Arc<ListenerSet<DialUpEvent<Value>>>,
    pub dial_rotate: Arc<ListenerSet<DialRotateEvent<Value>>>,
    pub touch_tap: Arc<ListenerSet<TouchTapEvent<Value>>>,
    pub title_parameters: Arc<ListenerSet<TitleParametersDidChangeEvent<Value>>>,
    pub did_receive_settings: Arc<ListenerSet<DidReceiveSettingsEvent<Value>>>,
    pub did_receive_resources: Arc<ListenerSet<DidReceiveResourcesEvent<Value>>>,
    pub did_receive_global_settings: Arc<ListenerSet<DidReceiveGlobalSettingsEvent<Value>>>,
    pub property_inspector_did_appear: Arc<ListenerSet<PropertyInspectorDidAppearEvent>>,
    pub property_inspector_did_disappear: Arc<ListenerSet<PropertyInspectorDidDisappearEvent>>,
    pub send_to_plugin: Arc<ListenerSet<SendToPluginEvent<Value>>>,
    pub application_did_launch: Arc<ListenerSet<ApplicationDidLaunchEvent>>,
    pub application_did_terminate: Arc<ListenerSet<ApplicationDidTerminateEvent>>,
    pub system_did_wake_up: Arc<ListenerSet<SystemDidWakeUpEvent>>,
    pub did_receive_deep_link: Arc<ListenerSet<DidReceiveDeepLinkEvent>>,
    pub device_did_connect: Arc<ListenerSet<DeviceDidConnectEvent>>,
    pub device_did_disconnect: Arc<ListenerSet<DeviceDidDisconnectEvent>>,
    pub device_did_change: Arc<ListenerSet<DeviceDidChangeEvent>>,
}

impl EventListeners {
    fn new() -> Self {
        Self {
            will_appear: Arc::new(ListenerSet::new()),
            will_disappear: Arc::new(ListenerSet::new()),
            key_down: Arc::new(ListenerSet::new()),
            key_up: Arc::new(ListenerSet::new()),
            dial_down: Arc::new(ListenerSet::new()),
            dial_up: Arc::new(ListenerSet::new()),
            dial_rotate: Arc::new(ListenerSet::new()),
            touch_tap: Arc::new(ListenerSet::new()),
            title_parameters: Arc::new(ListenerSet::new()),
            did_receive_settings: Arc::new(ListenerSet::new()),
            did_receive_resources: Arc::new(ListenerSet::new()),
            did_receive_global_settings: Arc::new(ListenerSet::new()),
            property_inspector_did_appear: Arc::new(ListenerSet::new()),
            property_inspector_did_disappear: Arc::new(ListenerSet::new()),
            send_to_plugin: Arc::new(ListenerSet::new()),
            application_did_launch: Arc::new(ListenerSet::new()),
            application_did_terminate: Arc::new(ListenerSet::new()),
            system_did_wake_up: Arc::new(ListenerSet::new()),
            did_receive_deep_link: Arc::new(ListenerSet::new()),
            device_did_connect: Arc::new(ListenerSet::new()),
            device_did_disconnect: Arc::new(ListenerSet::new()),
            device_did_change: Arc::new(ListenerSet::new()),
        }
    }
}

pub struct UiState {
    pub current_id: RwLock<Option<String>>,
    pub current_manifest: RwLock<Option<String>>,
    pub current_device: RwLock<Option<String>>,
    pub stack: AtomicI32,
}

impl UiState {
    fn new() -> Self {
        Self {
            current_id: RwLock::new(None),
            current_manifest: RwLock::new(None),
            current_device: RwLock::new(None),
            stack: AtomicI32::new(0),
        }
    }

    pub fn current_action_id(&self) -> Option<String> {
        self.current_id.read().expect("ui").clone()
    }

    pub fn is_current(&self, id: &str, manifest_id: &str, device_id: &str) -> bool {
        self.current_id.read().expect("ui").as_deref() == Some(id)
            && self.current_manifest.read().expect("ui").as_deref() == Some(manifest_id)
            && self.current_device.read().expect("ui").as_deref() == Some(device_id)
    }

    pub fn set_current(&self, id: String, manifest_id: String, device_id: String) {
        *self.current_id.write().expect("ui") = Some(id);
        *self.current_manifest.write().expect("ui") = Some(manifest_id);
        *self.current_device.write().expect("ui") = Some(device_id);
        self.stack.store(1, Ordering::SeqCst);
    }

    pub fn inc_stack(&self) {
        self.stack.fetch_add(1, Ordering::SeqCst);
    }

    pub fn dec_stack(&self) -> i32 {
        let v = self.stack.fetch_sub(1, Ordering::SeqCst) - 1;
        if v <= 0 {
            *self.current_id.write().expect("ui") = None;
            *self.current_manifest.write().expect("ui") = None;
            *self.current_device.write().expect("ui") = None;
            self.stack.store(0, Ordering::SeqCst);
        }
        v
    }
}

pub struct Runtime {
    pub registration: RegistrationParameters,
    pub version: Version,
    pub logger: Logger,
    outgoing_tx: mpsc::UnboundedSender<String>,
    outgoing_rx: Mutex<Option<mpsc::UnboundedReceiver<String>>>,
    pub events: broadcast::Sender<Arc<PluginEvent>>,
    pub action_store: ActionStore,
    pub device_store: DeviceStore,
    pub settings_cache: Mutex<HashMap<String, Value>>,
    experimental_ids: AtomicBool,
    pub(crate) registered: RwLock<Vec<Arc<dyn ErasedAction>>>,
    pub listeners: EventListeners,
    pub ui: UiState,
    pub manifest: Option<Manifest>,
}

impl Runtime {
    pub fn new(registration: RegistrationParameters) -> Arc<Self> {
        let version = Version::parse(&registration.info.application.version).unwrap_or(Version {
            major: 0,
            minor: 0,
            patch: 0,
            build: 0,
        });
        let uuid = {
            let from_cwd = plugin_uuid_from_cwd();
            if from_cwd == "plugin" && !registration.info.plugin.uuid.is_empty() {
                registration.info.plugin.uuid.clone()
            } else {
                from_cwd
            }
        };
        let logger = Logger::new(&uuid);
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(1024);
        let manifest = Manifest::load().ok();
        Arc::new(Self {
            registration,
            version,
            logger,
            outgoing_tx,
            outgoing_rx: Mutex::new(Some(outgoing_rx)),
            events,
            action_store: ActionStore::default(),
            device_store: DeviceStore::default(),
            settings_cache: Mutex::new(HashMap::new()),
            experimental_ids: AtomicBool::new(false),
            registered: RwLock::new(Vec::new()),
            listeners: EventListeners::new(),
            ui: UiState::new(),
            manifest,
        })
    }

    pub fn experimental_ids(&self) -> bool {
        self.experimental_ids.load(Ordering::SeqCst)
    }

    pub fn set_experimental_ids(&self, value: bool) {
        self.experimental_ids.store(value, Ordering::SeqCst);
    }

    pub fn take_outgoing_rx(&self) -> Option<mpsc::UnboundedReceiver<String>> {
        self.outgoing_rx.lock().expect("outgoing").take()
    }

    pub async fn send(&self, command: PluginCommand) -> Result<()> {
        let json = serde_json::to_string(&command)?;
        self.logger.create_scope("Connection").trace(&json);
        self.outgoing_tx
            .send(json)
            .map_err(|_| Error::Disconnected)?;
        Ok(())
    }

    pub async fn request(&self, command: PluginCommand) -> Result<Arc<PluginEvent>> {
        let context = match &command {
            PluginCommand::GetSettings { context, .. }
            | PluginCommand::GetResources { context, .. }
            | PluginCommand::GetGlobalSettings { context, .. }
            | PluginCommand::GetSecrets { context } => context.clone(),
            _ => String::new(),
        };
        let kind = match &command {
            PluginCommand::GetSettings { .. } => "didReceiveSettings",
            PluginCommand::GetResources { .. } => "didReceiveResources",
            PluginCommand::GetGlobalSettings { .. } => "didReceiveGlobalSettings",
            PluginCommand::GetSecrets { .. } => "didReceiveSecrets",
            _ => "",
        };
        let mut rx = self.events.subscribe();
        self.send(command).await?;
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        let matches = match ev.as_ref() {
                            PluginEvent::DidReceiveSettings(m) if kind == "didReceiveSettings" => {
                                m.context == context
                            }
                            PluginEvent::DidReceiveResources(m)
                                if kind == "didReceiveResources" =>
                            {
                                m.context == context
                            }
                            PluginEvent::DidReceiveGlobalSettings { .. }
                                if kind == "didReceiveGlobalSettings" =>
                            {
                                true
                            }
                            PluginEvent::DidReceiveSecrets { .. }
                                if kind == "didReceiveSecrets" =>
                            {
                                true
                            }
                            _ => false,
                        };
                        if matches {
                            return Ok(ev);
                        }
                    }
                    Err(_) => return Err(Error::Disconnected),
                }
            }
        })
        .await
        .map_err(|_| Error::Timeout)?
    }

    pub fn seed_devices(&self) {
        for dev in &self.registration.info.devices {
            if self.device_store.get(&dev.id).is_none() {
                self.device_store.set(crate::devices::Device::new(
                    dev.id.clone(),
                    dev.info(),
                    false,
                ));
            }
        }
    }

    pub fn plugin_uuid(&self) -> &str {
        &self.registration.plugin_uuid
    }
}
