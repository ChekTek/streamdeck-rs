use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Stream Deck device types.
///
/// Unknown integers are preserved so a new device (or a host-specific code such as
/// `-1`) cannot fail plugin registration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DeviceType {
    #[default]
    StreamDeck,
    StreamDeckMini,
    StreamDeckXL,
    StreamDeckMobile,
    CorsairGKeys,
    StreamDeckPedal,
    CorsairVoyager,
    StreamDeckPlus,
    ScufController,
    StreamDeckNeo,
    StreamDeckStudio,
    VirtualStreamDeck,
    Galleon100Sd,
    StreamDeckPlusXl,
    Unknown(i32),
}

impl DeviceType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::StreamDeck,
            1 => Self::StreamDeckMini,
            2 => Self::StreamDeckXL,
            3 => Self::StreamDeckMobile,
            4 => Self::CorsairGKeys,
            5 => Self::StreamDeckPedal,
            6 => Self::CorsairVoyager,
            7 => Self::StreamDeckPlus,
            8 => Self::ScufController,
            9 => Self::StreamDeckNeo,
            10 => Self::StreamDeckStudio,
            11 => Self::VirtualStreamDeck,
            12 => Self::Galleon100Sd,
            13 => Self::StreamDeckPlusXl,
            other => Self::Unknown(other),
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            Self::StreamDeck => 0,
            Self::StreamDeckMini => 1,
            Self::StreamDeckXL => 2,
            Self::StreamDeckMobile => 3,
            Self::CorsairGKeys => 4,
            Self::StreamDeckPedal => 5,
            Self::CorsairVoyager => 6,
            Self::StreamDeckPlus => 7,
            Self::ScufController => 8,
            Self::StreamDeckNeo => 9,
            Self::StreamDeckStudio => 10,
            Self::VirtualStreamDeck => 11,
            Self::Galleon100Sd => 12,
            Self::StreamDeckPlusXl => 13,
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for DeviceType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(self.as_i32())
    }
}

impl<'de> Deserialize<'de> for DeviceType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i32::deserialize(deserializer).map(Self::from_i32)
    }
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
///
/// Unknown names are preserved so a new controller type cannot fail event parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Controller {
    Keypad,
    Encoder,
    Unknown(String),
}

impl Controller {
    pub fn from_str_code(value: &str) -> Self {
        match value {
            "Keypad" => Self::Keypad,
            "Encoder" => Self::Encoder,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Keypad => "Keypad",
            Self::Encoder => "Encoder",
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for Controller {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Controller {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| Self::from_str_code(&value))
    }
}

/// Languages supported by Stream Deck.
///
/// Unknown codes are preserved so a new language cannot fail plugin registration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum Language {
    De,
    #[default]
    En,
    Es,
    Fr,
    Ja,
    Ko,
    ZhCn,
    ZhTw,
    Unknown(String),
}

impl Language {
    pub fn from_str_code(value: &str) -> Self {
        match value {
            "de" => Self::De,
            "en" => Self::En,
            "es" => Self::Es,
            "fr" => Self::Fr,
            "ja" => Self::Ja,
            "ko" => Self::Ko,
            "zh_CN" => Self::ZhCn,
            "zh_TW" => Self::ZhTw,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::De => "de",
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::ZhCn => "zh_CN",
            Self::ZhTw => "zh_TW",
            Self::Unknown(value) => value,
        }
    }

    pub fn primary(&self) -> &str {
        match self {
            Self::ZhCn | Self::ZhTw => "zh",
            Self::Unknown(code) => code.split(['-', '_']).next().unwrap_or(code),
            other => other.as_str(),
        }
    }
}

impl Serialize for Language {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Language {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| Self::from_str_code(&value))
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Operating system reported during registration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum Platform {
    #[default]
    Mac,
    Windows,
    Unknown(String),
}

impl Platform {
    pub fn from_str_code(value: &str) -> Self {
        match value {
            "mac" => Self::Mac,
            "windows" => Self::Windows,
            other => Self::Unknown(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Mac => "mac",
            Self::Windows => "windows",
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for Platform {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Platform {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| Self::from_str_code(&value))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn device_type_keeps_unknown_integers() {
        let value: DeviceType = serde_json::from_value(json!(-1)).unwrap();
        assert_eq!(value, DeviceType::Unknown(-1));
        assert_eq!(serde_json::to_value(value).unwrap(), json!(-1));
        assert_eq!(
            serde_json::from_value::<DeviceType>(json!(7)).unwrap(),
            DeviceType::StreamDeckPlus
        );
    }

    #[test]
    fn language_keeps_unknown_codes() {
        assert_eq!(
            serde_json::from_value::<Language>(json!("it")).unwrap(),
            Language::Unknown("it".into())
        );
        assert_eq!(
            serde_json::to_value(Language::Unknown("it".into())).unwrap(),
            json!("it")
        );
        assert_eq!(
            serde_json::from_value::<Language>(json!("zh_CN")).unwrap(),
            Language::ZhCn
        );
    }

    #[test]
    fn platform_keeps_unknown_codes() {
        assert_eq!(
            serde_json::from_value::<Platform>(json!("linux")).unwrap(),
            Platform::Unknown("linux".into())
        );
        assert_eq!(
            serde_json::from_value::<Platform>(json!("mac")).unwrap(),
            Platform::Mac
        );
    }

    #[test]
    fn controller_keeps_unknown_names() {
        assert_eq!(
            serde_json::from_value::<Controller>(json!("Touchscreen")).unwrap(),
            Controller::Unknown("Touchscreen".into())
        );
        assert_eq!(
            serde_json::from_value::<Controller>(json!("Keypad")).unwrap(),
            Controller::Keypad
        );
    }

    #[test]
    fn registration_info_accepts_unknown_device_type() {
        let info: RegistrationInfo = serde_json::from_value(json!({
            "application": { "language": "it", "platform": "linux", "version": "7.1.0" },
            "devices": [{
                "id": "nightsWord",
                "name": "NIGHTSWORD",
                "size": { "columns": 5, "rows": 3 },
                "type": -1
            }]
        }))
        .unwrap();
        assert_eq!(info.application.language, Language::Unknown("it".into()));
        assert_eq!(info.application.platform, Platform::Unknown("linux".into()));
        assert_eq!(info.devices[0].device_type, DeviceType::Unknown(-1));
    }
}
