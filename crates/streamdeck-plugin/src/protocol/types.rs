use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Stream Deck device types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DeviceType {
    #[default]
    StreamDeck = 0,
    StreamDeckMini = 1,
    StreamDeckXL = 2,
    StreamDeckMobile = 3,
    CorsairGKeys = 4,
    StreamDeckPedal = 5,
    CorsairVoyager = 6,
    StreamDeckPlus = 7,
    ScufController = 8,
    StreamDeckNeo = 9,
    StreamDeckStudio = 10,
    VirtualStreamDeck = 11,
    Galleon100Sd = 12,
    StreamDeckPlusXl = 13,
}

/// Hardware vs software update target for `setTitle` / `setImage`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Target {
    #[default]
    HardwareAndSoftware = 0,
    Hardware = 1,
    Software = 2,
}

/// Controller type for an action instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Controller {
    Keypad,
    Encoder,
}

/// Languages supported by Stream Deck.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "de")]
    De,
    #[default]
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es")]
    Es,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "zh_CN")]
    ZhCn,
    #[serde(rename = "zh_TW")]
    ZhTw,
}

impl Language {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::De => "de",
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::ZhCn => "zh_CN",
            Self::ZhTw => "zh_TW",
        }
    }

    pub fn primary(self) -> &'static str {
        match self {
            Self::ZhCn | Self::ZhTw => "zh",
            other => other.as_str(),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Operating system reported during registration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Mac,
    Windows,
}

/// Coordinates of an action on a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coordinates {
    pub column: u32,
    pub row: u32,
}

/// Device grid size, excluding dials / touchscreens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Size {
    pub columns: u32,
    pub rows: u32,
}

/// Information about a Stream Deck device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub name: String,
    pub size: Size,
    #[serde(rename = "type")]
    pub device_type: DeviceType,
}

/// Device entry included in registration info (includes `id`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrationDevice {
    pub id: String,
    pub name: String,
    pub size: Size,
    #[serde(rename = "type")]
    pub device_type: DeviceType,
}

impl RegistrationDevice {
    pub fn info(&self) -> DeviceInfo {
        DeviceInfo {
            name: self.name.clone(),
            size: self.size,
            device_type: self.device_type,
        }
    }
}

/// Resources (files) associated with an action.
pub type Resources = serde_json::Map<String, serde_json::Value>;

/// Action state index (0 or 1 for multi-state actions).
pub type State = u32;

/// Information about the Stream Deck application, plugin, OS, and devices.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationInfo {
    #[serde(default)]
    pub application: ApplicationInfo,
    #[serde(default)]
    pub colors: Colors,
    #[serde(default)]
    pub device_pixel_ratio: f64,
    #[serde(default)]
    pub devices: Vec<RegistrationDevice>,
    #[serde(default)]
    pub plugin: PluginInfo,
}

/// Registration info without the devices list.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub application: ApplicationInfo,
    pub colors: Colors,
    pub device_pixel_ratio: f64,
    pub plugin: PluginInfo,
}

impl RegistrationInfo {
    pub fn without_devices(&self) -> Info {
        Info {
            application: self.application.clone(),
            colors: self.colors.clone(),
            device_pixel_ratio: self.device_pixel_ratio,
            plugin: self.plugin.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationInfo {
    #[serde(default)]
    pub font: String,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub platform: Platform,
    #[serde(default)]
    pub platform_version: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Colors {
    #[serde(default)]
    pub button_mouse_over_background_color: String,
    #[serde(default)]
    pub button_pressed_background_color: String,
    #[serde(default)]
    pub button_pressed_border_color: String,
    #[serde(default)]
    pub button_pressed_text_color: String,
    #[serde(default)]
    pub highlight_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginInfo {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub version: String,
}
