use crate::error::Result;
use crate::events::{DeviceDidChangeEvent, DeviceDidConnectEvent, DeviceDidDisconnectEvent};
use crate::listeners::{Subscription, subscribe};
use crate::runtime::Runtime;
use crate::validation::requires_version;

mod device;
mod store;

pub use device::Device;
pub use store::DeviceStore;

use std::sync::Arc;

/// Functions and information for interacting with Stream Deck devices.
#[derive(Clone)]
pub struct DeviceService {
    pub(crate) runtime: Arc<Runtime>,
}

impl DeviceService {
    pub fn get_device_by_id(&self, id: &str) -> Option<Device> {
        self.runtime.device_store.get(id)
    }

    pub fn iter(&self) -> Vec<Device> {
        self.runtime.device_store.list()
    }

    pub fn on_device_did_change<F, Fut>(&self, listener: F) -> Result<Subscription>
    where
        F: Fn(DeviceDidChangeEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        requires_version(
            7,
            0,
            &self.runtime.version,
            "onDeviceDidChange",
            &self.runtime,
        )?;
        Ok(subscribe(
            &self.runtime.listeners.device_did_change,
            listener,
        ))
    }

    pub fn on_device_did_connect<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DeviceDidConnectEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.device_did_connect, listener)
    }

    pub fn on_device_did_disconnect<F, Fut>(&self, listener: F) -> Subscription
    where
        F: Fn(DeviceDidDisconnectEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        subscribe(&self.runtime.listeners.device_did_disconnect, listener)
    }
}
