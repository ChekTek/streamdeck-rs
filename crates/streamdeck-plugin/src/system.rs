use std::sync::Arc;

use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::error::Result;
use crate::events::{
    ApplicationDidLaunchEvent, ApplicationDidTerminateEvent, DidReceiveDeepLinkEvent,
    SystemDidWakeUpEvent,
};
use crate::listeners::{Subscription, subscribe};
use crate::protocol::PluginCommand;
use crate::runtime::Runtime;
use crate::validation::{requires_sdk_version, requires_version};

/// Interact with, and receive events from, the system the plugin is running on.
#[derive(Clone)]
pub struct SystemApi {
    pub(crate) runtime: Arc<Runtime>,
}

impl SystemApi {
    pub fn on_application_did_launch<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(ApplicationDidLaunchEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.application_did_launch, listener)
    }

    pub fn on_application_did_terminate<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(ApplicationDidTerminateEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.application_did_terminate, listener)
    }

    pub fn on_did_receive_deep_link<F, Fut>(&self, listener: F) -> Result<Subscription>
    where
        F: Fn(DidReceiveDeepLinkEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        requires_version(
            6,
            5,
            &self.runtime.version,
            "Receiving deep-link messages",
            &self.runtime,
        )?;
        Ok(subscribe(
            &self.runtime.listeners.did_receive_deep_link,
            listener,
        ))
    }

    pub fn on_system_did_wake_up<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(SystemDidWakeUpEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.system_did_wake_up, listener)
    }

    pub async fn open_url(&self, url: impl Into<String>) -> Result<()> {
        self.runtime
            .send(PluginCommand::OpenUrl {
                payload: crate::protocol::OpenUrlPayload { url: url.into() },
            })
            .await
    }

    pub async fn get_secrets<T: DeserializeOwned>(&self) -> Result<T> {
        requires_version(6, 9, &self.runtime.version, "Secrets", &self.runtime)?;
        requires_sdk_version(&self.runtime, 3, "Secrets")?;
        let ev = self
            .runtime
            .request(PluginCommand::GetSecrets {
                context: self.runtime.plugin_uuid().to_string(),
                id: Some(Uuid::new_v4().to_string()),
            })
            .await?;
        match ev.as_ref() {
            crate::protocol::PluginEvent::DidReceiveSecrets { payload, .. } => {
                Ok(serde_json::from_value(payload.secrets.clone())?)
            }
            _ => Err(crate::error::Error::Message(
                "unexpected secrets response".into(),
            )),
        }
    }
}
