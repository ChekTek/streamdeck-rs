use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rust_embed::RustEmbed;

use crate::stream_deck::PLUGIN_SUFFIX;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub author: String,
    pub name: String,
    pub uuid: String,
    pub description: String,
    pub crate_name: String,
}

pub fn sdk_dependency() -> String {
    if let Ok(path) = std::env::var("STREAMDECK_PLUGIN_PATH") {
        let escaped = path.replace('\\', "/").replace('"', "\\\"");
        format!("streamdeck-plugin = {{ path = \"{escaped}\" }}")
    } else {
        format!("streamdeck-plugin = \"{}\"", env!("CARGO_PKG_VERSION"))
    }
}

pub fn render_template(destination: &Path, info: &PluginInfo) -> Result<()> {
    if destination.exists() {
        bail!("directory already exists: {}", destination.display());
    }
    std::fs::create_dir_all(destination)
        .with_context(|| format!("create {}", destination.display()))?;

    let vars = substitutions(info);
    let plugin_folder = format!("{}{PLUGIN_SUFFIX}", info.uuid);

    for name in Templates::iter() {
        let file = Templates::get(name.as_ref()).context("missing embedded template")?;
        let dest = map_template_path(destination, name.as_ref(), &plugin_folder);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }

        let bytes = file.data.as_ref();
        if name.ends_with(".tmpl") || is_text_template(name.as_ref()) {
            let text = String::from_utf8(bytes.to_vec())
                .with_context(|| format!("template {name} is not utf-8"))?;
            std::fs::write(&dest, apply(&text, &vars))
                .with_context(|| format!("write {}", dest.display()))?;
        } else {
            std::fs::write(&dest, bytes).with_context(|| format!("write {}", dest.display()))?;
        }
    }

    Ok(())
}

fn map_template_path(destination: &Path, name: &str, plugin_folder: &str) -> PathBuf {
    let mut relative = name.to_string();
    if let Some(stripped) = relative.strip_suffix(".tmpl") {
        relative = stripped.to_string();
    }

    let mapped = if relative == "gitignore" {
        ".gitignore".to_string()
    } else if let Some(rest) = relative.strip_prefix("vscode/") {
        format!(".vscode/{rest}")
    } else if relative == "vscode" {
        ".vscode".to_string()
    } else if let Some(rest) = relative.strip_prefix("sdPlugin/") {
        if rest == "sdignore" {
            format!("{plugin_folder}/.sdignore")
        } else {
            format!("{plugin_folder}/{rest}")
        }
    } else {
        relative
    };

    destination.join(mapped)
}

fn is_text_template(name: &str) -> bool {
    name.ends_with(".tmpl")
        || name.ends_with(".rs")
        || name.ends_with(".html")
        || name.ends_with(".json")
        || name.ends_with("gitignore")
        || name.ends_with("sdignore")
}

fn substitutions(info: &PluginInfo) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("author".into(), info.author.clone());
    vars.insert("name".into(), info.name.clone());
    vars.insert("uuid".into(), info.uuid.clone());
    vars.insert("description".into(), info.description.clone());
    vars.insert("crate_name".into(), info.crate_name.clone());
    vars.insert("sdk_dep".into(), sdk_dependency());
    vars.insert("author_json".into(), json_string(&info.author));
    vars.insert("name_json".into(), json_string(&info.name));
    vars.insert("description_json".into(), json_string(&info.description));
    vars.insert("uuid_json".into(), json_string(&info.uuid));
    vars.insert("os_json".into(), host_os_json());
    vars
}

fn host_os_json() -> String {
    if cfg!(windows) {
        r#"[{ "Platform": "windows", "MinimumVersion": "10" }]"#.into()
    } else {
        r#"[{ "Platform": "mac", "MinimumVersion": "12" }]"#.into()
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
}

fn apply(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (key, value) in vars {
        let needle = format!("{{{{{key}}}}}");
        out = out.replace(&needle, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PluginInfo {
        PluginInfo {
            author: "Example".into(),
            name: "Hello World".into(),
            uuid: "com.example.hello-world".into(),
            description: "A test plugin.".into(),
            crate_name: "hello-world".into(),
        }
    }

    fn test_dest(name: &str) -> PathBuf {
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tmp-tests")
            .join(name);
        let _ = std::fs::remove_dir_all(&dest);
        dest
    }

    #[test]
    fn renders_expected_tree() {
        let dest = test_dest("hello-world");
        render_template(&dest, &sample()).expect("render template");

        assert!(dest.join("Cargo.toml").is_file());
        assert!(dest.join(".gitignore").is_file());
        assert!(dest.join(".vscode/settings.json").is_file());
        assert!(dest.join("src/main.rs").is_file());
        assert!(dest.join("src/actions/increment_counter.rs").is_file());
        let plugin = dest.join("com.example.hello-world.sdPlugin");
        assert!(plugin.join("manifest.json").is_file());
        assert!(plugin.join(".sdignore").is_file());
        assert!(plugin.join("ui/increment-counter.html").is_file());
        assert!(plugin.join("imgs/actions/counter/key.png").is_file());

        let cargo = std::fs::read_to_string(dest.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"hello-world\""));
        assert!(cargo.contains("streamdeck-plugin"));

        let action =
            std::fs::read_to_string(dest.join("src/actions/increment_counter.rs")).unwrap();
        assert!(action.contains("com.example.hello-world.increment"));

        let manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(plugin.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["UUID"], "com.example.hello-world");
        assert_eq!(manifest["Name"], "Hello World");
        assert_eq!(manifest["SDKVersion"], 3);
        assert!(manifest.get("Nodejs").is_none());
        assert_eq!(manifest["CodePathMac"], "bin/hello-world");
        assert_eq!(manifest["CodePathWin"], "bin/hello-world.exe");
        let os = manifest["OS"].as_array().expect("OS");
        assert_eq!(os.len(), 1);
        #[cfg(windows)]
        assert_eq!(os[0]["Platform"], "windows");
        #[cfg(not(windows))]
        assert_eq!(os[0]["Platform"], "mac");
    }

    #[test]
    fn json_escapes_quotes() {
        let dest = test_dest("quoted");
        let mut info = sample();
        info.name = r#"Say "hi""#.into();
        info.uuid = "com.example.quoted".into();
        render_template(&dest, &info).expect("render template");
        let raw = std::fs::read_to_string(dest.join("com.example.quoted.sdPlugin/manifest.json"))
            .unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(manifest["Name"], r#"Say "hi""#);
    }
}
