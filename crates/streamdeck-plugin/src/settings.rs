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

/// Persist global and action settings within Stream Deck.
#[derive(Clone)]
pub struct SettingsApi {
    pub(crate) runtime: Arc<Runtime>,
}

impl SettingsApi {
    /// Whether `on_did_receive_settings` / `on_did_receive_global_settings` also fire after
    /// `get_settings()` / `get_global_settings()`.
    ///
    /// - `false` (default): they only fire when settings change in the property inspector.
    ///   Requires Stream Deck 7.1 and `Software.MinimumVersion` 7.1.
    /// - `true`: they also fire for replies to `get_settings()` / `get_global_settings()`.
    ///
    /// Replaces `use_experimental_message_identifiers`, with inverted meaning.
    pub fn use_legacy_settings_behavior(&self) -> bool {
        self.runtime.legacy_settings_behavior()
    }

    /// Set [`Self::use_legacy_settings_behavior`]. Clears the action settings cache when
    /// the mode changes; re-applying the current mode is a no-op.
    /// Fails, leaving the value unchanged, when switching to the default behavior on a
    /// Stream Deck (or manifest minimum) older than 7.1.
    pub fn set_use_legacy_settings_behavior(&self, value: bool) -> Result<()> {
        self.runtime.set_legacy_settings_behavior(value)
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

    /// Fires when global settings change in the property inspector (and, with legacy
    /// settings behavior, after `get_global_settings()`).
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

    /// Fires when action settings change in the property inspector (and, with legacy
    /// settings behavior, after `get_settings()`).
    pub fn on_did_receive_settings<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(crate::events::DidReceiveSettingsEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.did_receive_settings, listener)
    }
}
