use std::sync::Arc;

use crate::error::Result;
use crate::protocol::{PluginCommand, SwitchToProfilePayload};
use crate::runtime::Runtime;
use crate::validation::requires_version;

/// Switch between profiles distributed with the plugin.
#[derive(Clone)]
pub struct ProfilesApi {
    pub(crate) runtime: Arc<Runtime>,
}

impl ProfilesApi {
    /// Switch the current profile on `device_id`. When `profile` is `None`, the previous profile is restored.
    pub async fn switch_to_profile(
        &self,
        device_id: impl Into<String>,
        profile: Option<String>,
        page: Option<u32>,
    ) -> Result<()> {
        if page.is_some() {
            requires_version(
                6,
                5,
                &self.runtime.version,
                "Switching to a profile page",
                &self.runtime,
            )?;
        }
        self.runtime
            .send(PluginCommand::SwitchToProfile {
                context: self.runtime.plugin_uuid().to_string(),
                device: device_id.into(),
                payload: SwitchToProfilePayload { profile, page },
            })
            .await
    }
}
