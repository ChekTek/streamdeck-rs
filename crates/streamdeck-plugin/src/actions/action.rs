use crate::devices::Device;
use crate::error::Result;
use crate::protocol::Controller;

use super::ActionContext;
use super::dial::DialAction;
use super::handle::ActionHandle;
use super::key::KeyAction;
use super::neo_infobar::NeoInfobarAction;

/// A visible action instance: a key, a dial, or a Stream Deck Neo infobar.
///
/// Controller-specific APIs (`show_alert`, `set_title`, `set_feedback`, ...) live on the
/// variants; match or use `as_key` / `as_dial` / `as_neo_infobar` to reach them.
#[derive(Clone)]
pub enum Action {
    Key(KeyAction),
    Dial(DialAction),
    NeoInfobar(NeoInfobarAction),
}

impl Action {
    fn handle(&self) -> &ActionHandle {
        match self {
            Self::Key(a) => &a.handle,
            Self::Dial(a) => &a.handle,
            Self::NeoInfobar(a) => &a.handle,
        }
    }

    pub fn id(&self) -> &str {
        &self.handle().id
    }

    pub fn manifest_id(&self) -> &str {
        &self.handle().manifest_id
    }

    pub fn device(&self) -> &Device {
        &self.handle().device
    }

    pub fn controller(&self) -> Controller {
        self.handle().controller.clone()
    }

    pub fn is_key(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    pub fn is_dial(&self) -> bool {
        matches!(self, Self::Dial(_))
    }

    pub fn is_neo_infobar(&self) -> bool {
        matches!(self, Self::NeoInfobar(_))
    }

    pub fn as_key(&self) -> Option<&KeyAction> {
        match self {
            Self::Key(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_dial(&self) -> Option<&DialAction> {
        match self {
            Self::Dial(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_neo_infobar(&self) -> Option<&NeoInfobarAction> {
        match self {
            Self::NeoInfobar(a) => Some(a),
            _ => None,
        }
    }

    pub fn context(&self) -> ActionContext {
        self.handle().context()
    }

    pub async fn set_settings<S: serde::Serialize>(&self, settings: S) -> Result<()> {
        self.handle().set_settings(settings).await
    }

    pub async fn get_settings<S: serde::de::DeserializeOwned>(&self) -> Result<S> {
        self.handle().get_settings().await
    }

    pub async fn send_to_property_inspector<S: serde::Serialize>(&self, payload: S) -> Result<()> {
        self.handle().send_to_property_inspector(payload).await
    }

    pub async fn set_resources(&self, resources: crate::protocol::Resources) -> Result<()> {
        self.handle().set_resources(resources).await
    }

    pub async fn get_resources(&self) -> Result<crate::protocol::Resources> {
        self.handle().get_resources().await
    }
}
