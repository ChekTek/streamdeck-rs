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
    file: File,
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

    pub fn trace(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Trace, message.as_ref());
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
        Ok(Self { path, file })
    }

    fn write(&mut self, line: &str) -> std::io::Result<()> {
        if let Ok(meta) = self.file.metadata()
            && meta.len() > MAX_SIZE
        {
            self.rotate()?;
        }
        self.file.write_all(line.as_bytes())?;
        Ok(())
    }

    fn rotate(&mut self) -> std::io::Result<()> {
        let _ = self.file.flush();
        for i in (1..MAX_FILE_COUNT).rev() {
            let from = self.rotated_path(i);
            let to = self.rotated_path(i + 1);
            if from.exists() {
                let _ = fs::rename(&from, &to);
            }
        }
        let first = self.rotated_path(1);
        let _ = fs::rename(&self.path, &first);
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_levels() {
        assert_eq!(LogLevel::parse("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::parse("WARN"), Some(LogLevel::Warn));
    }
}
