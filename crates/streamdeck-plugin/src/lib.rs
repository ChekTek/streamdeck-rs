//! Unofficial Stream Deck plugin SDK for Rust.
//!
//! Native plugin binaries parse launch arguments from the Stream Deck app, connect over
//! WebSocket, and route events to [`SingletonAction`] implementations.

mod actions;
mod connection;
mod devices;
mod error;
mod events;
mod i18n;
mod listeners;
mod logging;
mod manifest;
mod profiles;
mod protocol;
mod registration;
mod runtime;
mod settings;
mod system;
mod ui;
mod validation;
mod version;

pub use actions::{Action, ActionContext, ActionService, DialAction, KeyAction, SingletonAction};
pub use devices::{Device, DeviceService};
pub use error::{Error, Result};
pub use events::*;
pub use i18n::I18n;
pub use listeners::Subscription;
pub use logging::Logger;
pub use profiles::ProfilesApi;
pub use protocol::{
    ApplicationInfo, Bar, BarSubType, Colors, Controller, Coordinates, DeviceInfo, DeviceType,
    FeedbackPayload, GBar, ImageOptions, Info, Language, Pixmap, Platform, PluginCommand,
    PluginEvent, PluginInfo, RegistrationDevice, RegistrationInfo, Resources, Size, State, Target,
    Text, TitleOptions, TriggerDescription,
};
pub use registration::RegistrationParameters;
pub use settings::SettingsApi;
pub use system::SystemApi;
pub use ui::UiController;
pub use version::Version;

use std::sync::Arc;

use crate::runtime::{Runtime, set_runtime, try_runtime};

/// Plugin facade, equivalent to the TypeScript `streamDeck` object.
#[derive(Clone)]
pub struct StreamDeck {
    runtime: Arc<Runtime>,
}

impl StreamDeck {
    /// Parse registration arguments from `std::env::args`.
    pub fn new() -> Result<Self> {
        Self::from_args(std::env::args().skip(1))
    }

    /// Parse registration arguments from an iterator of flags.
    pub fn from_args<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let registration = RegistrationParameters::parse(args)?;
        let runtime = Runtime::new(registration);
        set_runtime(runtime.clone());
        Ok(Self { runtime })
    }

    pub fn register_action<A: SingletonAction>(self, action: A) -> Result<Self> {
        self.actions().register_action(action)?;
        Ok(self)
    }

    /// Connect to Stream Deck and run until the connection closes.
    pub async fn connect(self) -> Result<()> {
        connection::run(self.runtime).await
    }

    pub fn actions(&self) -> ActionService {
        ActionService {
            runtime: self.runtime.clone(),
        }
    }

    pub fn devices(&self) -> DeviceService {
        DeviceService {
            runtime: self.runtime.clone(),
        }
    }

    pub fn settings(&self) -> SettingsApi {
        SettingsApi {
            runtime: self.runtime.clone(),
        }
    }

    pub fn system(&self) -> SystemApi {
        SystemApi {
            runtime: self.runtime.clone(),
        }
    }

    pub fn ui(&self) -> UiController {
        UiController {
            runtime: self.runtime.clone(),
        }
    }

    pub fn profiles(&self) -> ProfilesApi {
        ProfilesApi {
            runtime: self.runtime.clone(),
        }
    }

    pub fn logger(&self) -> Logger {
        self.runtime.logger.clone()
    }

    pub fn info(&self) -> Info {
        self.runtime.registration.info.without_devices()
    }

    pub fn i18n(&self) -> I18n {
        I18n::from(self.runtime.as_ref())
    }

    pub fn registration(&self) -> &RegistrationParameters {
        &self.runtime.registration
    }
}

/// Process-wide logger (available after [`StreamDeck::new`]).
pub fn logger() -> Result<Logger> {
    Ok(try_runtime()?.logger.clone())
}

/// Process-wide actions namespace.
pub fn actions() -> Result<ActionService> {
    Ok(ActionService {
        runtime: try_runtime()?,
    })
}

/// Process-wide devices namespace.
pub fn devices() -> Result<DeviceService> {
    Ok(DeviceService {
        runtime: try_runtime()?,
    })
}

/// Process-wide settings namespace.
pub fn settings() -> Result<SettingsApi> {
    Ok(SettingsApi {
        runtime: try_runtime()?,
    })
}

/// Process-wide system namespace.
pub fn system() -> Result<SystemApi> {
    Ok(SystemApi {
        runtime: try_runtime()?,
    })
}

/// Process-wide UI namespace.
pub fn ui() -> Result<UiController> {
    Ok(UiController {
        runtime: try_runtime()?,
    })
}

/// Process-wide profiles namespace.
pub fn profiles() -> Result<ProfilesApi> {
    Ok(ProfilesApi {
        runtime: try_runtime()?,
    })
}

/// Registration info without devices.
pub fn info() -> Result<Info> {
    Ok(try_runtime()?.registration.info.without_devices())
}

/// Process-wide i18n provider.
pub fn i18n() -> Result<I18n> {
    Ok(I18n::from(try_runtime()?.as_ref()))
}
