use crate::devices::Device;
use crate::error::Result;
use crate::protocol::Controller;

use super::ActionContext;
use super::dial::DialAction;
use super::key::KeyAction;

/// A visible action instance, either a key or a dial.
#[derive(Clone)]
pub enum Action {
    Key(KeyAction),
    Dial(DialAction),
}

impl Action {
    pub fn id(&self) -> &str {
        match self {
            Self::Key(a) => a.id(),
            Self::Dial(a) => a.id(),
        }
    }

    pub fn manifest_id(&self) -> &str {
        match self {
            Self::Key(a) => a.manifest_id(),
            Self::Dial(a) => a.manifest_id(),
        }
    }

    pub fn device(&self) -> &Device {
        match self {
            Self::Key(a) => a.device(),
            Self::Dial(a) => a.device(),
        }
    }

    pub fn controller(&self) -> Controller {
        match self {
            Self::Key(a) => a.handle.controller.clone(),
            Self::Dial(a) => a.handle.controller.clone(),
        }
    }

    pub fn is_key(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    pub fn is_dial(&self) -> bool {
        matches!(self, Self::Dial(_))
    }

    pub fn as_key(&self) -> Option<&KeyAction> {
        match self {
            Self::Key(a) => Some(a),
            Self::Dial(_) => None,
        }
    }

    pub fn as_dial(&self) -> Option<&DialAction> {
        match self {
            Self::Dial(a) => Some(a),
            Self::Key(_) => None,
        }
    }

    pub fn context(&self) -> ActionContext {
        match self {
            Self::Key(a) => a.context(),
            Self::Dial(a) => a.context(),
        }
    }

    pub async fn set_settings<S: serde::Serialize>(&self, settings: S) -> Result<()> {
        match self {
            Self::Key(a) => a.set_settings(settings).await,
            Self::Dial(a) => a.set_settings(settings).await,
        }
    }

    pub async fn get_settings<S: serde::de::DeserializeOwned>(&self) -> Result<S> {
        match self {
            Self::Key(a) => a.get_settings().await,
            Self::Dial(a) => a.get_settings().await,
        }
    }

    pub async fn show_alert(&self) -> Result<()> {
        match self {
            Self::Key(a) => a.show_alert().await,
            Self::Dial(a) => a.show_alert().await,
        }
    }

    pub async fn send_to_property_inspector<S: serde::Serialize>(&self, payload: S) -> Result<()> {
        match self {
            Self::Key(a) => a.send_to_property_inspector(payload).await,
            Self::Dial(a) => a.send_to_property_inspector(payload).await,
        }
    }

    pub async fn set_resources(&self, resources: crate::protocol::Resources) -> Result<()> {
        match self {
            Self::Key(a) => a.set_resources(resources).await,
            Self::Dial(a) => a.set_resources(resources).await,
        }
    }

    pub async fn get_resources(&self) -> Result<crate::protocol::Resources> {
        match self {
            Self::Key(a) => a.get_resources().await,
            Self::Dial(a) => a.get_resources().await,
        }
    }
}
