use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde_json::Value;

use crate::error::{Error, Result};
use crate::protocol::Language;
use crate::runtime::Runtime;

/// File-based localization provider (`{lang}.json` with a `Localization` object).
#[derive(Clone)]
pub struct I18n {
    language: Arc<RwLock<Language>>,
    cache: Arc<RwLock<HashMap<String, Option<Value>>>>,
}

impl I18n {
    pub(crate) fn new(language: Language) -> Self {
        Self {
            language: Arc::new(RwLock::new(language)),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn language(&self) -> Language {
        self.language.read().expect("i18n").clone()
    }

    pub fn set_language(&self, language: Language) {
        *self.language.write().expect("i18n") = language;
    }

    pub fn t(&self, key: &str) -> String {
        self.translate(key, self.language())
    }

    pub fn translate(&self, key: &str, language: Language) -> String {
        let langs = [language.as_str(), language.primary(), Language::En.as_str()];
        for lang in langs {
            if let Some(resource) = self.lookup(lang, key) {
                return resource;
            }
        }
        key.to_string()
    }

    fn lookup(&self, language: &str, key: &str) -> Option<String> {
        let translations = self.translations(language)?;
        dotted_get(&translations, key).map(|v| match v {
            Value::String(s) => s,
            other => other.to_string(),
        })
    }

    fn translations(&self, language: &str) -> Option<Value> {
        {
            let cache = self.cache.read().expect("i18n");
            if let Some(hit) = cache.get(language) {
                return hit.clone();
            }
        }
        let loaded = load_locale(language);
        self.cache
            .write()
            .expect("i18n")
            .insert(language.to_string(), loaded.clone());
        loaded
    }
}

impl From<&Runtime> for I18n {
    fn from(runtime: &Runtime) -> Self {
        Self::new(runtime.registration.info.application.language.clone())
    }
}

fn load_locale(language: &str) -> Option<Value> {
    let path = std::env::current_dir()
        .ok()?
        .join(format!("{language}.json"));
    if !path.exists() {
        return None;
    }
    let text = std::fs::read_to_string(&path).ok()?;
    parse_localizations(&text).ok()
}

pub fn parse_localizations(contents: &str) -> Result<Value> {
    let json: Value = serde_json::from_str(contents)?;
    match json.get("Localization") {
        Some(loc) if loc.is_object() => Ok(loc.clone()),
        _ => Err(Error::InvalidLocalizations),
    }
}

fn dotted_get(source: &Value, path: &str) -> Option<Value> {
    let mut cur = source;
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_localization_object() {
        assert!(parse_localizations(r#"{"hello":"nope"}"#).is_err());
        let v = parse_localizations(r#"{"Localization":{"hello":"world","nested":{"a":"b"}}}"#)
            .unwrap();
        assert_eq!(dotted_get(&v, "hello").unwrap(), "world");
        assert_eq!(dotted_get(&v, "nested.a").unwrap(), "b");
    }
}
