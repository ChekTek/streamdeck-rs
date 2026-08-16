use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

const MAX_FILE_COUNT: usize = 10;
const MAX_SIZE: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

struct LoggerShared {
    level: LogLevel,
    console: bool,
    file: Mutex<Option<FileTarget>>,
}

struct FileTarget {
    path: PathBuf,
    file: Option<File>,
}

/// Logger matching the TypeScript SDK: file under `cwd/logs/{pluginUUID}.log`,
/// plus console when `STREAMDECK_LOG` or `RUST_LOG` is set.
#[derive(Clone)]
pub struct Logger {
    name: String,
    shared: Arc<LoggerShared>,
}

impl Logger {
    pub fn new(plugin_uuid: &str) -> Self {
        let env_level = std::env::var("STREAMDECK_LOG")
            .ok()
            .or_else(|| std::env::var("RUST_LOG").ok());
        let console = env_level.is_some();
        let level = env_level
            .as_deref()
            .and_then(|v| v.split(',').next())
            .and_then(|v| {
                let v = v.split('=').next_back().unwrap_or(v);
                LogLevel::parse(v)
            })
            .unwrap_or(if console {
                LogLevel::Debug
            } else {
                LogLevel::Info
            });

        let file = FileTarget::open(plugin_uuid).ok();
        Self {
            name: String::new(),
            shared: Arc::new(LoggerShared {
                level,
                console,
                file: Mutex::new(file),
            }),
        }
    }

    pub fn create_scope(&self, name: &str) -> Self {
        let name = if self.name.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", self.name, name)
        };
        Self {
            name,
            shared: self.shared.clone(),
        }
    }

    pub fn enabled(&self, level: LogLevel) -> bool {
        level >= self.shared.level
    }

    pub fn trace(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Trace, message.as_ref());
    }

    /// Build a TRACE message only when that level will actually be emitted.
    pub fn trace_with(&self, message: impl FnOnce() -> String) {
        if self.enabled(LogLevel::Trace) {
            self.log(LogLevel::Trace, &message());
        }
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Debug, message.as_ref());
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Info, message.as_ref());
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Warn, message.as_ref());
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Error, message.as_ref());
    }

    fn log(&self, level: LogLevel, message: &str) {
        if level < self.shared.level {
            return;
        }
        let name = if self.name.is_empty() {
            "streamdeck"
        } else {
            self.name.as_str()
        };
        let line = format!("{} {} {}: {}\n", timestamp(), level.as_str(), name, message);
        if self.shared.console {
            eprint!("{line}");
        }
        if let Ok(mut guard) = self.shared.file.lock()
            && let Some(target) = guard.as_mut()
        {
            let _ = target.write(&line);
        }
    }
}

impl FileTarget {
    fn open(plugin_uuid: &str) -> std::io::Result<Self> {
        let dir = Path::new("logs");
        fs::create_dir_all(dir)?;
        let path = dir.join(format!("{plugin_uuid}.log"));
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file: Some(file),
        })
    }

    fn write(&mut self, line: &str) -> std::io::Result<()> {
        let should_rotate = self
            .ensure_open()?
            .metadata()
            .map(|meta| meta.len() > MAX_SIZE)
            .unwrap_or(false);
        if should_rotate {
            self.rotate()?;
        }
        self.ensure_open()?.write_all(line.as_bytes())?;
        Ok(())
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
            drop(file);
        }
        let oldest = self.rotated_path(MAX_FILE_COUNT);
        if oldest.exists() {
            fs::remove_file(&oldest)?;
        }
        for i in (1..MAX_FILE_COUNT).rev() {
            let from = self.rotated_path(i);
            if from.exists() {
                fs::rename(&from, self.rotated_path(i + 1))?;
            }
        }
        if self.path.exists() {
            fs::rename(&self.path, self.rotated_path(1))?;
        }
        self.file = Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?,
        );
        Ok(())
    }

    fn ensure_open(&mut self) -> std::io::Result<&mut File> {
        if self.file.is_none() {
            self.file = Some(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)?,
            );
        }
        Ok(self.file.as_mut().expect("log file"))
    }

    fn rotated_path(&self, n: usize) -> PathBuf {
        self.path.with_extension(format!("log.{n}"))
    }
}

fn timestamp() -> String {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}.{:03}Z", d.as_secs(), d.subsec_millis()),
        Err(_) => "0".into(),
    }
}

/// Plugin UUID derived from the `.sdPlugin` folder name, matching the TS SDK.
pub fn plugin_uuid_from_cwd() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .map(|name| {
            name.strip_suffix(".sdPlugin")
                .map(ToString::to_string)
                .unwrap_or(name)
        })
        .unwrap_or_else(|| "plugin".into())
}

/// Log file stem: `.sdPlugin` folder name, then `-info` plugin UUID, then `-pluginUUID`.
pub(crate) fn log_file_stem(
    plugin_uuid_flag: Option<&str>,
    info_plugin_uuid: Option<&str>,
) -> String {
    let from_cwd = plugin_uuid_from_cwd();
    if from_cwd != "plugin" {
        return from_cwd;
    }
    if let Some(id) = info_plugin_uuid.filter(|s| !s.is_empty()) {
        return id.to_string();
    }
    if let Some(id) = plugin_uuid_flag.filter(|s| !s.is_empty()) {
        return id.to_string();
    }
    "plugin".into()
}

/// Redact secrets and `setImage` payloads before tracing WebSocket JSON.
pub(crate) fn redact_for_log(text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(mut value) => {
            let event = value
                .get("event")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned);
            if event.as_deref() == Some("didReceiveSecrets")
                && let Some(payload) = value
                    .get_mut("payload")
                    .and_then(serde_json::Value::as_object_mut)
            {
                payload.insert(
                    "secrets".into(),
                    serde_json::Value::String("[redacted]".into()),
                );
            }
            if event.as_deref() == Some("setImage")
                && let Some(payload) = value
                    .get_mut("payload")
                    .and_then(serde_json::Value::as_object_mut)
                && payload
                    .get("image")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
            {
                payload.insert(
                    "image".into(),
                    serde_json::Value::String("[redacted]".into()),
                );
            }
            value.to_string()
        }
        Err(_) if text.contains("didReceiveSecrets") => "[redacted didReceiveSecrets]".into(),
        Err(_) => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logger_with_level(level: LogLevel) -> Logger {
        Logger {
            name: String::new(),
            shared: Arc::new(LoggerShared {
                level,
                console: false,
                file: Mutex::new(None),
            }),
        }
    }

    #[test]
    fn parses_levels() {
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("WARN"), Some(LogLevel::Warn));
    }

    #[test]
    fn trace_with_skips_closure_when_disabled() {
        let logger = logger_with_level(LogLevel::Info);
        let called = std::sync::atomic::AtomicBool::new(false);
        logger.trace_with(|| {
            called.store(true, std::sync::atomic::Ordering::SeqCst);
            "expensive".into()
        });
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!logger.enabled(LogLevel::Trace));
        assert!(logger.enabled(LogLevel::Info));
    }

    #[test]
    fn redacts_secrets_and_set_image() {
        let secrets = r#"{"event":"didReceiveSecrets","payload":{"secrets":{"token":"s3cret"}}}"#;
        let redacted_secrets = redact_for_log(secrets);
        assert!(!redacted_secrets.contains("s3cret"));
        assert!(redacted_secrets.contains("[redacted]"));

        let image = r#"{"event":"setImage","context":"c","payload":{"image":"data:image/png;base64,AAAA"}}"#;
        let redacted_image = redact_for_log(image);
        assert!(!redacted_image.contains("AAAA"));
        assert!(redacted_image.contains("[redacted]"));
        assert!(redacted_image.contains("setImage"));
    }

    #[test]
    fn rotate_removes_oldest_backup_and_shifts_files() {
        let dir = std::env::temp_dir().join(format!(
            "sd-log-rotate-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("plugin.log");
        fs::write(&path, b"current").unwrap();
        for i in 1..=MAX_FILE_COUNT {
            fs::write(path.with_extension(format!("log.{i}")), format!("old-{i}")).unwrap();
        }

        let file = OpenOptions::new().append(true).open(&path).unwrap();
        let mut target = FileTarget {
            path: path.clone(),
            file: Some(file),
        };
        target.rotate().unwrap();

        assert_eq!(
            fs::read_to_string(dir.join("plugin.log.1")).unwrap(),
            "current"
        );
        assert_eq!(
            fs::read_to_string(dir.join("plugin.log.2")).unwrap(),
            "old-1"
        );
        assert_eq!(
            fs::read_to_string(dir.join("plugin.log.10")).unwrap(),
            "old-9"
        );
        assert!(path.is_file());
        assert_eq!(fs::read_to_string(&path).unwrap(), "");

        let _ = fs::remove_dir_all(&dir);
    }
}
