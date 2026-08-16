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

/// Default Tokio worker stack size. Rasterizing key images (for example with
/// `resvg`) on a worker can overflow the 2 MiB default and SIGSEGV.
pub const PLUGIN_THREAD_STACK_SIZE: usize = 8 * 1024 * 1024;

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
    ///
    /// Opens `logs/{pluginUUID}.log` before parsing `-info` so registration failures
    /// are written to the plugin log instead of looking like a silent crash.
    pub fn from_args<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        let plugin_uuid = crate::registration::flag_value(&args, "-pluginUUID");
        let logger = Logger::new(&crate::logging::log_file_stem(plugin_uuid.as_deref(), None));
        let registration = match RegistrationParameters::parse(args) {
            Ok(registration) => registration,
            Err(err) => {
                logger.error(err.to_string());
                return Err(err);
            }
        };
        let runtime = Runtime::with_logger(registration, logger);
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

/// Run a plugin future on a multi-thread Tokio runtime with an 8 MiB worker stack.
///
/// Prefer [`tokio::task::spawn_blocking`] for heavy CPU work inside action handlers
/// (`resvg`, font shaping, image encode). This helper is a safety net for the
/// process so those stacks are less likely to overflow.
pub fn block_on<T, E>(
    fut: impl Future<Output = std::result::Result<T, E>>,
) -> std::result::Result<T, E>
where
    E: From<std::io::Error>,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(PLUGIN_THREAD_STACK_SIZE)
        .build()?
        .block_on(fut)
}
