use crate::devices::Device;
use crate::error::Result;
use crate::protocol::{Coordinates, ImageOptions, State, TitleOptions};

use super::ActionContext;
use super::handle::ActionHandle;

/// A keypad (key / pedal / G-key) action instance.
#[derive(Clone)]
pub struct KeyAction {
    pub(crate) handle: ActionHandle,
    coordinates: Option<Coordinates>,
    is_in_multi_action: bool,
}

impl KeyAction {
    pub fn id(&self) -> &str {
        &self.handle.id
    }

    pub fn manifest_id(&self) -> &str {
        &self.handle.manifest_id
    }

    pub fn device(&self) -> &Device {
        &self.handle.device
    }

    pub fn coordinates(&self) -> Option<Coordinates> {
        self.coordinates
    }

    pub fn is_in_multi_action(&self) -> bool {
        self.is_in_multi_action
    }

    pub fn is_key(&self) -> bool {
        true
    }

    pub fn is_dial(&self) -> bool {
        false
    }

    pub fn context(&self) -> ActionContext {
        ActionContext {
            id: self.handle.id.clone(),
            manifest_id: self.handle.manifest_id.clone(),
            device: self.handle.device.clone(),
            controller: self.handle.controller,
        }
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

    pub async fn show_alert(&self) -> Result<()> {
        self.handle.show_alert().await
    }

    pub async fn send_to_property_inspector<S: serde::Serialize>(&self, payload: S) -> Result<()> {
        self.handle.send_to_property_inspector(payload).await
    }

    pub async fn set_image(&self, image: impl Into<String>) -> Result<()> {
        self.set_image_opts(Some(image.into()), ImageOptions::default())
            .await
    }

    pub async fn set_image_opts(&self, image: Option<String>, options: ImageOptions) -> Result<()> {
        self.handle.set_image_inner(image, options).await
    }

    pub async fn set_title(&self, title: impl Into<String>) -> Result<()> {
        self.set_title_opts(Some(title.into()), TitleOptions::default())
            .await
    }

    pub async fn set_title_opts(&self, title: Option<String>, options: TitleOptions) -> Result<()> {
        self.handle.set_title_inner(title, options).await
    }

    pub async fn set_state(&self, state: State) -> Result<()> {
        self.handle.set_state_inner(state).await
    }

    pub async fn show_ok(&self) -> Result<()> {
        self.handle.show_ok_inner().await
    }

    pub(crate) fn new(
        handle: ActionHandle,
        coordinates: Option<Coordinates>,
        is_in_multi_action: bool,
    ) -> Self {
        Self {
            handle,
            coordinates,
            is_in_multi_action,
        }
    }
}
