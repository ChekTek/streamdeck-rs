use crate::devices::Device;
use crate::error::Result;
use crate::protocol::{Coordinates, FeedbackPayload, TriggerDescription};

use super::ActionContext;
use super::handle::ActionHandle;

/// A dial / encoder action instance (Stream Deck +).
#[derive(Clone)]
pub struct DialAction {
    pub(crate) handle: ActionHandle,
    coordinates: Coordinates,
}

impl DialAction {
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
        true
    }

    pub fn context(&self) -> ActionContext {
        ActionContext {
            id: self.handle.id.clone(),
            manifest_id: self.handle.manifest_id.clone(),
            device: self.handle.device.clone(),
            controller: self.handle.controller.clone(),
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

    pub async fn set_feedback(&self, feedback: impl serde::Serialize) -> Result<()> {
        let value = serde_json::to_value(feedback)?;
        let serde_json::Value::Object(payload) = value else {
            return Err(crate::error::Error::Message(
                "set_feedback payload must be a JSON object".into(),
            ));
        };
        self.handle
            .runtime
            .send(crate::protocol::PluginCommand::SetFeedback {
                context: self.handle.id.clone(),
                payload,
            })
            .await
    }

    pub async fn set_feedback_layout(&self, layout: impl Into<String>) -> Result<()> {
        self.handle
            .runtime
            .send(crate::protocol::PluginCommand::SetFeedbackLayout {
                context: self.handle.id.clone(),
                payload: crate::protocol::SetFeedbackLayoutPayload {
                    layout: layout.into(),
                },
            })
            .await
    }

    pub async fn set_image(&self, image: Option<impl Into<String>>) -> Result<()> {
        self.handle
            .set_image_inner(image.map(Into::into), Default::default())
            .await
    }

    /// Dial titles are applied via layout feedback (`setFeedback({ title })`).
    pub async fn set_title(&self, title: impl Into<String>) -> Result<()> {
        let mut payload = FeedbackPayload::new();
        payload.insert("title".into(), serde_json::Value::String(title.into()));
        self.set_feedback(payload).await
    }

    pub async fn set_trigger_description(
        &self,
        descriptions: Option<TriggerDescription>,
    ) -> Result<()> {
        self.handle
            .runtime
            .send(crate::protocol::PluginCommand::SetTriggerDescription {
                context: self.handle.id.clone(),
                payload: descriptions.unwrap_or_default(),
            })
            .await
    }

    pub(crate) fn new(handle: ActionHandle, coordinates: Coordinates) -> Self {
        Self {
            handle,
            coordinates,
        }
    }
}
