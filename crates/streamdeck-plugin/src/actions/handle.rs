use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::devices::Device;
use crate::error::{Error, Result};
use crate::protocol::{
    Controller, PluginCommand, PluginEvent, Resources, SetImagePayload, SetTitlePayload, State,
};
use crate::runtime::Runtime;
use crate::validation::requires_version;

/// Shared identity + command sender for a visible action instance.
#[derive(Clone)]
pub struct ActionHandle {
    pub(crate) runtime: Arc<Runtime>,
    pub id: String,
    pub manifest_id: String,
    pub device: Device,
    pub controller: Controller,
}

impl ActionHandle {
    pub async fn set_settings<S: Serialize>(&self, settings: S) -> Result<()> {
        self.runtime
            .settings_cache
            .lock()
            .expect("settings cache")
            .remove(&self.id);
        self.runtime
            .send(PluginCommand::SetSettings {
                context: self.id.clone(),
                payload: serde_json::to_value(settings)?,
            })
            .await
    }

    pub async fn get_settings<S: DeserializeOwned>(&self) -> Result<S> {
        if self.runtime.experimental_ids() {
            let cache = self.runtime.settings_cache.lock().expect("settings cache");
            if let Some(cached) = cache.get(&self.id) {
                self.runtime.logger.create_scope("Connection").trace(format!(
                    r#"{{"event":"getSettings","context":"{}","source":"cache","settings":{}}}"#,
                    self.id, cached
                ));
                return Ok(serde_json::from_value(cached.clone())?);
            }
        }

        let value = self
            .runtime
            .request(PluginCommand::GetSettings {
                context: self.id.clone(),
                id: Some(Uuid::new_v4().to_string()),
            })
            .await?;
        match value.as_ref() {
            PluginEvent::DidReceiveSettings(m) => {
                Ok(serde_json::from_value(m.payload.settings.clone())?)
            }
            _ => Err(Error::Message("unexpected settings response".into())),
        }
    }

    pub async fn set_resources(&self, resources: Resources) -> Result<()> {
        requires_version(7, 1, &self.runtime.version, "setResources", &self.runtime)?;
        self.runtime
            .send(PluginCommand::SetResources {
                context: self.id.clone(),
                payload: resources,
            })
            .await
    }

    pub async fn get_resources(&self) -> Result<Resources> {
        requires_version(7, 1, &self.runtime.version, "getResources", &self.runtime)?;
        let value = self
            .runtime
            .request(PluginCommand::GetResources {
                context: self.id.clone(),
                id: Some(Uuid::new_v4().to_string()),
            })
            .await?;
        match value.as_ref() {
            PluginEvent::DidReceiveResources(m) => Ok(m.payload.resources.clone()),
            _ => Err(Error::Message("unexpected resources response".into())),
        }
    }

    pub async fn show_alert(&self) -> Result<()> {
        self.runtime
            .send(PluginCommand::ShowAlert {
                context: self.id.clone(),
            })
            .await
    }

    pub async fn send_to_property_inspector<S: Serialize>(&self, payload: S) -> Result<()> {
        self.runtime
            .send(PluginCommand::SendToPropertyInspector {
                context: self.id.clone(),
                payload: serde_json::to_value(payload)?,
            })
            .await
    }

    pub(crate) async fn set_title_inner(
        &self,
        title: Option<String>,
        options: SetTitlePayload,
    ) -> Result<()> {
        let mut payload = options;
        payload.title = title;
        self.runtime
            .send(PluginCommand::SetTitle {
                context: self.id.clone(),
                payload,
            })
            .await
    }

    pub(crate) async fn set_image_inner(
        &self,
        image: Option<String>,
        options: SetImagePayload,
    ) -> Result<()> {
        let mut payload = options;
        payload.image = image;
        self.runtime
            .send(PluginCommand::SetImage {
                context: self.id.clone(),
                payload,
            })
            .await
    }

    pub(crate) async fn set_state_inner(&self, state: State) -> Result<()> {
        self.runtime
            .send(PluginCommand::SetState {
                context: self.id.clone(),
                payload: crate::protocol::SetStatePayload { state },
            })
            .await
    }

    pub(crate) async fn show_ok_inner(&self) -> Result<()> {
        self.runtime
            .send(PluginCommand::ShowOk {
                context: self.id.clone(),
            })
            .await
    }

    #[allow(dead_code)]
    pub fn is_dial(&self) -> bool {
        self.controller == Controller::Encoder
    }

    #[allow(dead_code)]
    pub fn is_key(&self) -> bool {
        self.controller == Controller::Keypad
    }
}
