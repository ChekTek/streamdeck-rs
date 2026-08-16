use std::sync::Arc;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::events::*;
use crate::listeners::{Subscription, subscribe};
use crate::runtime::Runtime;

use super::{Action, ErasedAction, SingletonAction};

/// Namespace for event listeners and functionality relating to Stream Deck actions.
#[derive(Clone)]
pub struct ActionService {
    pub(crate) runtime: Arc<Runtime>,
}

impl ActionService {
    pub fn get_action_by_id(&self, id: &str) -> Option<Action> {
        self.runtime.action_store.get(id)
    }

    pub fn iter(&self) -> Vec<Action> {
        self.runtime.action_store.list()
    }

    pub fn register_action<A: SingletonAction>(&self, action: A) -> Result<()> {
        if A::UUID.is_empty() {
            return Err(Error::MissingActionUuid);
        }
        if let Some(manifest) = &self.runtime.manifest
            && !manifest.actions.iter().any(|a| a.uuid == A::UUID)
        {
            return Err(Error::ActionNotInManifest(A::UUID.to_string()));
        }
        self.runtime
            .registered
            .write()
            .expect("registered actions")
            .push(Arc::new(action) as Arc<dyn ErasedAction>);
        Ok(())
    }

    pub fn on_key_down<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(KeyDownEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.key_down, listener)
    }

    pub fn on_key_up<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(KeyUpEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.key_up, listener)
    }

    pub fn on_dial_down<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DialDownEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.dial_down, listener)
    }

    pub fn on_dial_up<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DialUpEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.dial_up, listener)
    }

    pub fn on_dial_rotate<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DialRotateEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.dial_rotate, listener)
    }

    pub fn on_touch_tap<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(TouchTapEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.touch_tap, listener)
    }

    pub fn on_will_appear<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(WillAppearEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.will_appear, listener)
    }

    pub fn on_will_disappear<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(WillDisappearEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.will_disappear, listener)
    }

    pub fn on_title_parameters_did_change<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(TitleParametersDidChangeEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.title_parameters, listener)
    }

    pub fn on_did_receive_resources<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DidReceiveResourcesEvent<Value>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.did_receive_resources, listener)
    }
}
