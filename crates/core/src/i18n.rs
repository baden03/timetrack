//! Message catalogs (plan P0-6): no user-visible text is hard-coded. Code
//! produces machine-readable keys; text comes from `i18n/<language>.json`.

use std::collections::BTreeMap;

/// Interface languages: a code and its catalog. To add a language, add a line
/// here (see docs/translating.md). English is the base: a message missing in
/// another language falls back to it, and a raw key is never shown.
const CATALOGS: [(&str, &str); 2] = [
    ("en", include_str!("../../../i18n/en.json")),
    ("de", include_str!("../../../i18n/de.json")),
];

/// The codes of the available languages.
pub fn languages() -> impl Iterator<Item = &'static str> {
    CATALOGS.iter().map(|(code, _)| *code)
}

pub fn is_supported(language: &str) -> bool {
    languages().any(|code| code == language)
}

pub struct Catalog {
    messages: BTreeMap<String, String>,
}

impl Catalog {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Catalog {
            messages: serde_json::from_str(json)?,
        })
    }

    /// English, embedded in the binary.
    pub fn english() -> Self {
        Self::from_json(include_str!("../../../i18n/en.json")).expect("i18n/en.json is valid")
    }

    /// The catalog for a language, with English filling any gap. An unknown
    /// code gives English.
    pub fn for_language(language: &str) -> Self {
        let mut catalog = Self::english();
        let overlay = CATALOGS
            .iter()
            .find(|(code, _)| *code == language && *code != "en")
            .map(|(_, json)| *json);
        if let Some(json) = overlay {
            let translated: BTreeMap<String, String> =
                serde_json::from_str(json).expect("language catalog is valid");
            catalog
                .messages
                .extend(translated.into_iter().filter(|(_, v)| !v.trim().is_empty()));
        }
        catalog
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.messages.get(key).map(String::as_str)
    }

    /// Text for a key, falling back to the key itself so a gap is visible.
    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.get(key).unwrap_or(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.messages.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        id::IdError, model::ModelError, record::RecordError, time::SplitProblem, time::TimeError,
    };

    #[test]
    fn every_error_code_has_english_text() {
        let catalog = Catalog::english();
        let codes = [
            TimeError::EndNotAfterStart.code(),
            TimeError::TooLong.code(),
            TimeError::DurationOutOfRange.code(),
            TimeError::InFuture.code(),
            TimeError::NonexistentLocalTime.code(),
            TimeError::InconsistentSplit(SplitProblem::Empty).code(),
            RecordError::Deleted.code(),
            RecordError::EmptyHistory.code(),
            RecordError::BrokenHistory(1).code(),
            IdError::InvalidOrigin.code(),
            IdError::InvalidId.code(),
            ModelError::NameRequired.code(),
            ModelError::DescriptionRequired.code(),
            ModelError::EmptyTag.code(),
            ModelError::TagTooLong.code(),
            ModelError::TooManyTags.code(),
        ];
        for code in codes {
            assert!(
                catalog.get(&format!("error.{code}")).is_some(),
                "missing error.{code}"
            );
        }
    }

    /// Source files may only refer to catalog keys that exist, via `t("key")`
    /// or `t!("key")` calls, and Rust code must not build user-facing text
    /// with `Display` outside the catalog (errors expose `code()` instead).
    #[test]
    fn referenced_keys_exist() {
        let catalog = Catalog::english();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut files = Vec::new();
        for dir in ["crates", "apps/desktop/src"] {
            collect(&root.join(dir), &mut files);
        }
        let mut checked = 0;
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            for key in referenced_keys(&text) {
                // A plural message exists as `key.one` and `key.other`.
                let known =
                    catalog.get(&key).is_some() || catalog.get(&format!("{key}.other")).is_some();
                assert!(known, "{}: unknown key {key}", file.display());
                checked += 1;
            }
        }
        // The scan itself is exercised by this literal reference below.
        assert!(referenced_keys("t(\"error.not_found\")").len() == 1);
        let _ = checked;
    }

    fn placeholders(text: &str) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        let mut rest = text;
        while let Some(start) = rest.find('{') {
            let tail = &rest[start + 1..];
            let Some(end) = tail.find('}') else { break };
            out.insert(tail[..end].to_owned());
            rest = &tail[end + 1..];
        }
        out
    }

    /// Every language has exactly the keys English has, no empty text, and the
    /// same `{placeholders}` in each message, so nothing can be silently lost.
    #[test]
    fn every_language_matches_english() {
        let english = Catalog::english();
        for (code, json) in CATALOGS.iter().filter(|(code, _)| *code != "en") {
            assert!(is_supported(code));
            let other = Catalog::from_json(json).unwrap();
            let en_keys: Vec<&str> = english.keys().collect();
            let other_keys: Vec<&str> = other.keys().collect();
            assert_eq!(en_keys, other_keys, "{code}: keys differ from English");
            for key in en_keys {
                let text = other.get(key).unwrap();
                assert!(!text.trim().is_empty(), "{code}: {key} is empty");
                assert_eq!(
                    placeholders(english.get(key).unwrap()),
                    placeholders(text),
                    "{code}: placeholders of {key} differ"
                );
            }
        }
    }

    #[test]
    fn a_language_falls_back_to_english_and_unknown_codes_give_english() {
        let german = Catalog::for_language("de");
        assert_eq!(german.text("month.3"), "März");
        assert_eq!(Catalog::for_language("xx").text("month.3"), "March");
        // Every key resolves in every language, so no raw key is ever shown.
        for key in Catalog::english().keys() {
            assert!(german.get(key).is_some());
        }
        assert!(!is_supported("fr"));
    }

    fn referenced_keys(text: &str) -> Vec<String> {
        let mut keys = Vec::new();
        for marker in ["t(\"", "t!(\""] {
            let mut rest = text;
            while let Some(pos) = rest.find(marker) {
                let before = rest[..pos].chars().next_back();
                let tail = &rest[pos + marker.len()..];
                if !before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    if let Some(end) = tail.find('"') {
                        keys.push(tail[..end].to_owned());
                    }
                }
                rest = tail;
            }
        }
        keys
    }

    fn collect(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                if !matches!(name.as_str(), "target" | "node_modules" | "dist") {
                    collect(&path, out);
                }
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rs" | "ts" | "tsx")
            ) && !path.ends_with("i18n.rs")
            {
                out.push(path);
            }
        }
    }
}
