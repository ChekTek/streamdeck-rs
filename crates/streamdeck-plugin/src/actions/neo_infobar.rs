use crate::devices::Device;
use crate::error::Result;
use crate::protocol::Coordinates;

use super::ActionContext;
use super::handle::ActionHandle;

/// An infobar action instance (Stream Deck Neo).
///
/// Infobars render a layout like dials do, but have no alert indicator, title, or image.
#[derive(Clone)]
pub struct NeoInfobarAction {
    pub(crate) handle: ActionHandle,
    coordinates: Coordinates,
}

impl NeoInfobarAction {
    pub fn id(&self) -> &str {
        &self.handle.id
    }

    pub fn manifest_id(&self) -> &str {
        &self.handle.manifest_id
    }

    pub fn device(&self) -> &Device {
        &self.handle.device
    }

    pub fn coordinates(&self) -> Coordinates {
        self.coordinates
    }

    pub fn is_key(&self) -> bool {
        false
    }

    pub fn is_dial(&self) -> bool {
        false
    }

    pub fn is_neo_infobar(&self) -> bool {
        true
    }

    pub fn context(&self) -> ActionContext {
        self.handle.context()
    }

    pub async fn set_settings<S: serde::Serialize>(&self, settings: S) -> Result<()> {
        self.handle.set_settings(settings).await
    }

    pub async fn get_settings<S: serde::de::DeserializeOwned>(&self) -> Result<S> {
        self.handle.get_settings().await
    }

    pub async fn set_resources(&self, resources: crate::protocol::Resources) -> Result<()> {
        self.handle.set_resources(resources).await
    }

    pub async fn get_resources(&self) -> Result<crate::protocol::Resources> {
        self.handle.get_resources().await
    }

    pub async fn send_to_property_inspector<S: serde::Serialize>(&self, payload: S) -> Result<()> {
        self.handle.send_to_property_inspector(payload).await
    }

    /// Update items in the current layout. Keys are layout item `key`s.
    pub async fn set_feedback(&self, feedback: impl serde::Serialize) -> Result<()> {
        self.handle.set_feedback_inner(feedback).await
    }

    /// Set the layout from a JSON file relative to the plugin folder. Infobars have no
    /// built-in layouts.
    pub async fn set_feedback_layout(&self, layout: impl Into<String>) -> Result<()> {
        self.handle.set_feedback_layout_inner(layout.into()).await
    }

    pub(crate) fn new(handle: ActionHandle, coordinates: Coordinates) -> Self {
        Self {
            handle,
            coordinates,
        }
    }
}
