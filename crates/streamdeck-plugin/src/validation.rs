use crate::error::{Error, Result};
use crate::runtime::Runtime;
use crate::version::Version;

/// Validate Stream Deck application version and manifest `Software.MinimumVersion`.
pub fn requires_version(
    major: u32,
    minor: u32,
    stream_deck_version: &Version,
    feature: &str,
    runtime: &Runtime,
) -> Result<()> {
    let required = Version {
        major,
        minor,
        patch: 0,
        build: 0,
    };
    let required_s = required.as_major_minor();
    if stream_deck_version.compare_to(&required) < 0 {
        return Err(Error::NotSupported {
            feature: feature.to_string(),
            required: required_s.clone(),
            current: stream_deck_version.as_major_minor(),
        });
    }
    if let Some(min) = runtime
        .manifest
        .as_ref()
        .and_then(|m| m.software_minimum_version())
        && min.compare_to(&required) < 0
    {
        return Err(Error::NotSupported {
            feature: feature.to_string(),
            required: required_s,
            current: min.as_major_minor(),
        });
    }
    Ok(())
}

/// Validate manifest `SDKVersion`.
pub fn requires_sdk_version(runtime: &Runtime, minimum: u32, feature: &str) -> Result<()> {
    if let Some(actual) = runtime.manifest.as_ref().and_then(|m| m.sdk_version)
        && minimum > actual
    {
        return Err(Error::SdkVersionNotSupported {
            feature: feature.to_string(),
            required: minimum,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::protocol::RegistrationInfo;
    use crate::registration::RegistrationParameters;

    use std::sync::Arc;

    fn runtime_with_version(ver: &str) -> Arc<Runtime> {
        let info = serde_json::from_str::<RegistrationInfo>(&format!(
            r#"{{"application":{{"version":"{ver}"}},"plugin":{{"uuid":"x","version":"1"}}}}"#
        ))
        .unwrap();
        let params = RegistrationParameters {
            port: "1".into(),
            plugin_uuid: "u".into(),
            register_event: "registerPlugin".into(),
            info,
        };
        Runtime::new(params)
    }

    #[test]
    fn rejects_old_stream_deck() {
        let rt = runtime_with_version("6.4");
        let err =
            requires_version(6, 5, &rt.version, "Receiving deep-link messages", &rt).unwrap_err();
        assert!(err.to_string().contains("ERR_NOT_SUPPORTED"));
    }

    #[test]
    fn accepts_current_version() {
        let rt = runtime_with_version("7.1");
        requires_version(6, 5, &rt.version, "deep-link", &rt).unwrap();
    }

    #[test]
    fn reports_manifest_minimum_version_when_that_check_fails() {
        let mut rt = runtime_with_version("7.1");
        Arc::get_mut(&mut rt).unwrap().manifest =
            Some(serde_json::from_str(r#"{"Software":{"MinimumVersion":"6.6"}}"#).unwrap());
        let err = requires_version(7, 0, &rt.version, "onDeviceDidChange", &rt).unwrap_err();
        match err {
            Error::NotSupported {
                required, current, ..
            } => {
                assert_eq!(required, "7.0");
                assert_eq!(current, "6.6");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
