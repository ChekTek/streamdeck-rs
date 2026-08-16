use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use uuid::Uuid;

use crate::error::Result;
use crate::events::DidReceiveGlobalSettingsEvent;
use crate::listeners::{Subscription, subscribe};
use crate::protocol::PluginCommand;
use crate::runtime::Runtime;
use crate::validation::requires_version;

/// Persist global and action settings within Stream Deck.
#[derive(Clone)]
pub struct SettingsApi {
    pub(crate) runtime: Arc<Runtime>,
}

impl SettingsApi {
    pub fn use_experimental_message_identifiers(&self) -> bool {
        self.runtime.experimental_ids()
    }

    pub fn set_use_experimental_message_identifiers(&self, value: bool) -> Result<()> {
        requires_version(
            7,
            1,
            &self.runtime.version,
            "Message identifiers",
            &self.runtime,
        )?;
        self.runtime.set_experimental_ids(value);
        Ok(())
    }

    pub async fn get_global_settings<T: DeserializeOwned>(&self) -> Result<T> {
        let ev = self
            .runtime
            .request(PluginCommand::GetGlobalSettings {
                context: self.runtime.plugin_uuid().to_string(),
                id: Some(Uuid::new_v4().to_string()),
            })
            .await?;
        match ev.as_ref() {
            crate::protocol::PluginEvent::DidReceiveGlobalSettings { payload, .. } => {
                Ok(serde_json::from_value(payload.settings.clone())?)
            }
            _ => Err(crate::error::Error::Message(
                "unexpected global settings response".into(),
            )),
        }
    }

    pub async fn set_global_settings<T: Serialize>(&self, settings: T) -> Result<()> {
        self.runtime
            .send(PluginCommand::SetGlobalSettings {
                context: self.runtime.plugin_uuid().to_string(),
                payload: serde_json::to_value(settings)?,
            })
            .await
    }

    pub fn on_did_receive_global_settings<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DidReceiveGlobalSettingsEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(
            &self.runtime.listeners.did_receive_global_settings,
            listener,
        )
    }

    pub fn on_did_receive_settings<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(crate::events::DidReceiveSettingsEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.did_receive_settings, listener)
    }
}
