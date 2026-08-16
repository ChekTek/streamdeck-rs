use crate::devices::Device;
use crate::protocol::Controller;

/// Identity of an action instance (used for `willDisappear` after the live handle is gone).
#[derive(Debug, Clone)]
pub struct ActionContext {
    pub id: String,
    pub manifest_id: String,
    pub device: Device,
    pub controller: Controller,
}

impl ActionContext {
    pub fn controller_type(&self) -> Controller {
        self.controller
    }
}
