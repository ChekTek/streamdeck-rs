use std::sync::{Arc, RwLock};

use crate::protocol::{DeviceInfo, DeviceType, Size};

/// A Stream Deck device known to the plugin.
#[derive(Clone)]
pub struct Device {
    inner: Arc<RwLock<DeviceInner>>,
}

struct DeviceInner {
    id: String,
    info: DeviceInfo,
    is_connected: bool,
}

impl Device {
    pub(crate) fn new(id: impl Into<String>, info: DeviceInfo, is_connected: bool) -> Self {
        Self {
            inner: Arc::new(RwLock::new(DeviceInner {
                id: id.into(),
                info,
                is_connected,
            })),
        }
    }

    pub fn id(&self) -> String {
        self.inner.read().expect("device").id.clone()
    }

    pub fn name(&self) -> String {
        self.inner.read().expect("device").info.name.clone()
    }

    pub fn size(&self) -> Size {
        self.inner.read().expect("device").info.size
    }

    pub fn device_type(&self) -> DeviceType {
        self.inner.read().expect("device").info.device_type
    }

    pub fn is_connected(&self) -> bool {
        self.inner.read().expect("device").is_connected
    }

    pub(crate) fn set_connected(&self, connected: bool) {
        self.inner.write().expect("device").is_connected = connected;
    }

    pub(crate) fn set_info(&self, info: DeviceInfo) {
        self.inner.write().expect("device").info = info;
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("id", &self.id())
            .field("name", &self.name())
            .field("connected", &self.is_connected())
            .finish()
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Device {}
