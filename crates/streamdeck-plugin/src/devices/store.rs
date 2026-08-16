use std::collections::HashMap;
use std::sync::RwLock;

use super::Device;

#[derive(Default)]
pub struct DeviceStore {
    items: RwLock<HashMap<String, Device>>,
}

impl DeviceStore {
    pub fn get(&self, id: &str) -> Option<Device> {
        self.items.read().expect("device store").get(id).cloned()
    }

    pub fn set(&self, device: Device) {
        self.items
            .write()
            .expect("device store")
            .insert(device.id(), device);
    }

    pub fn list(&self) -> Vec<Device> {
        self.items
            .read()
            .expect("device store")
            .values()
            .cloned()
            .collect()
    }
}
