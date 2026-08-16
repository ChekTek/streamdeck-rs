use crate::error::{Error, Result};
use crate::protocol::RegistrationInfo;

const PORT: &str = "-port";
const PLUGIN_UUID: &str = "-pluginUUID";
const REGISTER_EVENT: &str = "-registerEvent";
const INFO: &str = "-info";

/// Launch arguments supplied by Stream Deck so the plugin can register.
#[derive(Debug, Clone)]
pub struct RegistrationParameters {
    pub port: String,
    pub plugin_uuid: String,
    pub register_event: String,
    pub info: RegistrationInfo,
}

impl RegistrationParameters {
    /// Parse flags from an argv iterator (typically `std::env::args().skip(1)`).
    pub fn parse<I, S>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();

        let mut port = None;
        let mut plugin_uuid = None;
        let mut register_event = None;
        let mut info = None;

        let mut i = 0;
        while i + 1 < args.len() {
            let param = args[i].as_str();
            let value = args[i + 1].as_str();
            match param {
                PORT => {
                    port = Some(value.to_string());
                    i += 2;
                }
                PLUGIN_UUID => {
                    plugin_uuid = Some(value.to_string());
                    i += 2;
                }
                REGISTER_EVENT => {
                    register_event = Some(value.to_string());
                    i += 2;
                }
                INFO => {
                    info = Some(parse_info(value)?);
                    i += 2;
                }
                _ => i += 1,
            }
        }

        let mut missing = Vec::new();
        if port.is_none() {
            missing.push(PORT);
        }
        if plugin_uuid.is_none() {
            missing.push(PLUGIN_UUID);
        }
        if register_event.is_none() {
            missing.push(REGISTER_EVENT);
        }
        if info.is_none() {
            missing.push(INFO);
        }
        if !missing.is_empty() {
            return Err(Error::missing_args(&missing));
        }

        Ok(Self {
            port: port.unwrap(),
            plugin_uuid: plugin_uuid.unwrap(),
            register_event: register_event.unwrap(),
            info: info.unwrap(),
        })
    }

    pub fn from_env() -> Result<Self> {
        Self::parse(std::env::args().skip(1))
    }

    pub fn websocket_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }
}

fn parse_info(value: &str) -> Result<RegistrationInfo> {
    serde_json::from_str(value).map_err(Error::InvalidRegistrationInfo)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info() -> String {
        r#"{"plugin":{"uuid":"com.elgato.test","version":"0.1.0"}}"#.into()
    }

    #[test]
    fn parses_required_flags() {
        let params = RegistrationParameters::parse([
            "-port",
            "12345",
            "-pluginUUID",
            "abc123",
            "-registerEvent",
            "registerPlugin",
            "-info",
            &sample_info(),
        ])
        .unwrap();
        assert_eq!(params.port, "12345");
        assert_eq!(params.plugin_uuid, "abc123");
        assert_eq!(params.register_event, "registerPlugin");
        assert_eq!(params.info.plugin.uuid, "com.elgato.test");
    }

    #[test]
    fn skips_unknown_args() {
        let params = RegistrationParameters::parse([
            "plugin",
            "--verbose",
            "-port",
            "9",
            "-pluginUUID",
            "u",
            "-registerEvent",
            "registerPlugin",
            "-info",
            &sample_info(),
        ])
        .unwrap();
        assert_eq!(params.port, "9");
    }

    #[test]
    fn errors_when_missing() {
        let err = RegistrationParameters::parse(["-port", "1"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("-pluginUUID"));
        assert!(msg.contains("-registerEvent"));
        assert!(msg.contains("-info"));
    }
}
