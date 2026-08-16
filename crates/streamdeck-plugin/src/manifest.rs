use serde::Deserialize;

use crate::error::{Error, Result};
use crate::version::Version;

/// Subset of `manifest.json` required at runtime.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Manifest {
    #[serde(default, rename = "UUID")]
    pub uuid: Option<String>,
    #[serde(default, rename = "Version")]
    pub version: Option<String>,
    #[serde(default, rename = "SDKVersion")]
    pub sdk_version: Option<u32>,
    #[serde(default, rename = "Actions")]
    pub actions: Vec<ManifestAction>,
    #[serde(default, rename = "Software")]
    pub software: Option<Software>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ManifestAction {
    #[serde(rename = "UUID")]
    pub uuid: String,
    #[serde(default, rename = "Name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Software {
    pub minimum_version: Option<String>,
}

impl Manifest {
    pub fn load() -> Result<Self> {
        let path = std::env::current_dir()?.join("manifest.json");
        if !path.exists() {
            return Err(Error::ManifestMissing);
        }
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text).map_err(Error::InvalidManifest)
    }

    pub fn software_minimum_version(&self) -> Option<Version> {
        self.software
            .as_ref()
            .and_then(|s| s.minimum_version.as_deref())
            .and_then(|v| Version::parse(v).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_manifest() {
        let json = r#"{
            "UUID": "com.example.test",
            "SDKVersion": 2,
            "Actions": [{ "UUID": "com.example.test.one", "Name": "One" }],
            "Software": { "MinimumVersion": "6.6" }
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.uuid.as_deref(), Some("com.example.test"));
        assert_eq!(m.sdk_version, Some(2));
        assert_eq!(m.actions[0].uuid, "com.example.test.one");
        assert_eq!(m.software_minimum_version().unwrap().major, 6);
    }
}
