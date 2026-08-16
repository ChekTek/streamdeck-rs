use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::actions::Action;
use crate::error::Result;
use crate::events::{
    PropertyInspectorDidAppearEvent, PropertyInspectorDidDisappearEvent, SendToPluginEvent,
};
use crate::listeners::{Subscription, subscribe};
use crate::protocol::PluginCommand;
use crate::runtime::Runtime;

/// Controller for sending/receiving payloads with the property inspector.
#[derive(Clone)]
pub struct UiController {
    pub(crate) runtime: Arc<Runtime>,
}

impl UiController {
    /// Action associated with the current property inspector, if visible.
    pub fn action(&self) -> Option<Action> {
        self.runtime
            .ui
            .current_action_id()
            .and_then(|id| self.runtime.action_store.get(&id))
    }

    pub fn on_did_appear<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(PropertyInspectorDidAppearEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(
            &self.runtime.listeners.property_inspector_did_appear,
            listener,
        )
    }

    pub fn on_did_disappear<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(PropertyInspectorDidDisappearEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(
            &self.runtime.listeners.property_inspector_did_disappear,
            listener,
        )
    }

    pub fn on_send_to_plugin<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(SendToPluginEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.send_to_plugin, listener)
    }

    /// Send a payload to the currently visible property inspector.
    pub async fn send_to_property_inspector<S: Serialize>(&self, payload: S) -> Result<()> {
        if let Some(action) = self.action() {
            self.runtime
                .send(PluginCommand::SendToPropertyInspector {
                    context: action.id().to_string(),
                    payload: serde_json::to_value(payload)?,
                })
                .await?;
        }
        Ok(())
    }
}
