//! Typed payloads of the records a stand-alone installation keeps. They
//! serialize into `Record::payload`. Field names are the stored names.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::id::Id;

pub const KIND_CLIENT: &str = "client";
pub const KIND_PROJECT: &str = "project";
pub const KIND_CATEGORY: &str = "category";
pub const KIND_ENTRY: &str = "entry";
pub const KIND_SETTINGS: &str = "settings";
pub const KIND_TIMER: &str = "timer";

pub const MAX_TAGS: usize = 10;
pub const MAX_TAG_LENGTH: usize = 40;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ModelError {
    #[error("name required")]
    NameRequired,
    #[error("description required")]
    DescriptionRequired,
    #[error("empty tag")]
    EmptyTag,
    #[error("tag too long")]
    TagTooLong,
    #[error("too many tags")]
    TooManyTags,
    #[error("invalid color")]
    InvalidColor,
}

impl ModelError {
    /// Machine-readable code; the catalog entry is `error.<code>`.
    pub fn code(&self) -> &'static str {
        match self {
            ModelError::NameRequired => "name_required",
            ModelError::DescriptionRequired => "description_required",
            ModelError::EmptyTag => "tag_empty",
            ModelError::TagTooLong => "tag_too_long",
            ModelError::TooManyTags => "too_many_tags",
            ModelError::InvalidColor => "invalid_color",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client {
    pub name: String,
    /// Optional `#rrggbb` shown on cards and lists; the interface adjusts it
    /// to the light or dark theme.
    #[serde(default)]
    pub color: Option<String>,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub client_id: Id,
    pub name: String,
    /// Optional `#rrggbb`; falls back to the client's color.
    #[serde(default)]
    pub color: Option<String>,
    pub description: String,
    pub billable_default: bool,
    /// Categories enabled for this project (concept §7.4).
    pub category_ids: Vec<Id>,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub language: String,
    /// IANA zone name used for new entries.
    pub timezone: String,
    pub date_format: String,
    pub clock_24h: bool,
    pub theme: Theme,
    pub entry_window_days: u32,
    /// Days without a backup before the reminder appears.
    #[serde(default = "default_backup_reminder_days")]
    pub backup_reminder_days: u32,
}

fn default_backup_reminder_days() -> u32 {
    7
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Interval,
    Duration,
}

/// One stored piece of a time entry. An entry that crosses midnight is
/// several pieces sharing `group_id` (concept §8.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntry {
    pub project_id: Id,
    /// Optional: an entry may have no category.
    #[serde(default)]
    pub category_id: Option<Id>,
    pub description: String,
    pub billable: bool,
    pub tags: Vec<String>,
    pub kind: EntryKind,
    /// Local calendar date in `timezone`.
    pub work_date: NaiveDate,
    pub timezone: String,
    pub minutes: i64,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub start_offset: Option<i32>,
    pub end_offset: Option<i32>,
    pub group_id: String,
    pub piece_index: u32,
    pub piece_count: u32,
    pub span_start_at: Option<DateTime<Utc>>,
    pub span_end_at: Option<DateTime<Utc>>,
}

/// The running timer (concept §9). At most one exists per person. Its start
/// is captured to the minute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timer {
    pub project_id: Id,
    /// Chosen at the latest when the timer stops.
    #[serde(default)]
    pub category_id: Option<Id>,
    pub description: String,
    /// Running notes added while the timer runs, in order; they become bullet
    /// points of the entry's description when it stops.
    #[serde(default)]
    pub notes: Vec<String>,
    pub billable: Option<bool>,
    pub tags: Vec<String>,
    /// The start to the minute: what the entry will use.
    pub start_at: DateTime<Utc>,
    /// The moment the timer was actually started, for the elapsed display
    /// (the window and the menu bar both count from here).
    #[serde(default)]
    pub exact_start_at: Option<DateTime<Utc>>,
    /// The zone in force when the timer started.
    pub timezone: String,
}

/// `#rrggbb` (any case) or nothing; anything else is refused.
pub fn normalize_color(color: Option<&str>) -> Result<Option<String>, ModelError> {
    let Some(raw) = color.map(str::trim).filter(|c| !c.is_empty()) else {
        return Ok(None);
    };
    let ok =
        raw.len() == 7 && raw.starts_with('#') && raw[1..].bytes().all(|b| b.is_ascii_hexdigit());
    if ok {
        Ok(Some(raw.to_ascii_lowercase()))
    } else {
        Err(ModelError::InvalidColor)
    }
}

/// Builds an entry's description from the pieces typed while a timer ran and
/// at the stop. One piece stays as it is; several become bullet points.
pub fn compose_description(items: &[String]) -> Result<String, ModelError> {
    let items: Vec<&str> = items
        .iter()
        .map(|i| i.trim())
        .filter(|i| !i.is_empty())
        .collect();
    match items.as_slice() {
        [] => Err(ModelError::DescriptionRequired),
        [one] => Ok((*one).to_owned()),
        many => Ok(many
            .iter()
            .map(|i| format!("- {i}"))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

/// The separate items of a description: the lines of a bullet list, or the
/// whole text (line breaks and runs of spaces become single spaces) otherwise.
pub fn description_items(description: &str) -> Vec<String> {
    let lines: Vec<&str> = description
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let bullets = lines.len() >= 2 && lines.iter().all(|l| l.starts_with("- "));
    if bullets {
        lines.iter().map(|l| l[2..].trim().to_owned()).collect()
    } else {
        vec![description.split_whitespace().collect::<Vec<_>>().join(" ")]
    }
}

pub fn require_name(name: &str) -> Result<String, ModelError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Err(ModelError::NameRequired)
    } else {
        Ok(trimmed.to_owned())
    }
}

pub fn require_description(text: &str) -> Result<String, ModelError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Err(ModelError::DescriptionRequired)
    } else {
        Ok(trimmed.to_owned())
    }
}

/// Trims tags, rejects empty or over-long ones and repeats ignoring case
/// (the first spelling wins), and enforces the limit of ten (concept §7.5).
pub fn normalize_tags(tags: &[String]) -> Result<Vec<String>, ModelError> {
    let mut out: Vec<String> = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            return Err(ModelError::EmptyTag);
        }
        if tag.chars().count() > MAX_TAG_LENGTH {
            return Err(ModelError::TagTooLong);
        }
        if !out.iter().any(|t| t.to_lowercase() == tag.to_lowercase()) {
            out.push(tag.to_owned());
        }
    }
    if out.len() > MAX_TAGS {
        return Err(ModelError::TooManyTags);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn tags_are_trimmed_and_deduplicated_ignoring_case() {
        assert_eq!(
            normalize_tags(&tags(&[" Design ", "design", "Review"])).unwrap(),
            ["Design", "Review"]
        );
    }

    #[test]
    fn tag_limits() {
        assert_eq!(normalize_tags(&tags(&["  "])), Err(ModelError::EmptyTag));
        assert_eq!(
            normalize_tags(&[("x".repeat(41))]),
            Err(ModelError::TagTooLong)
        );
        assert!(normalize_tags(&[("x".repeat(40))]).is_ok());
        let eleven: Vec<String> = (0..11).map(|i| format!("t{i}")).collect();
        assert_eq!(normalize_tags(&eleven), Err(ModelError::TooManyTags));
        assert!(normalize_tags(&eleven[..10]).is_ok());
    }

    #[test]
    fn colors_are_hex_or_nothing() {
        assert_eq!(normalize_color(None), Ok(None));
        assert_eq!(normalize_color(Some("  ")), Ok(None));
        assert_eq!(normalize_color(Some("#1F6FEB")), Ok(Some("#1f6feb".into())));
        for bad in ["red", "#12345", "#1234567", "1f6feb", "#gggggg"] {
            assert_eq!(
                normalize_color(Some(bad)),
                Err(ModelError::InvalidColor),
                "{bad}"
            );
        }
    }

    #[test]
    fn running_notes_become_bullets_and_back() {
        let items = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            compose_description(&items(&["Only one"])).unwrap(),
            "Only one"
        );
        assert_eq!(
            compose_description(&items(&["Setup folders", " ", "Add files", "Update text"]))
                .unwrap(),
            "- Setup folders\n- Add files\n- Update text"
        );
        assert_eq!(
            compose_description(&items(&["  ", ""])),
            Err(ModelError::DescriptionRequired)
        );
        assert_eq!(
            description_items("- Setup folders\n- Add files\n- Update text"),
            ["Setup folders", "Add files", "Update text"]
        );
        // Anything that is not a bullet list is one item, on one line.
        assert_eq!(
            description_items("Fixed Safari\nlayout   bug"),
            ["Fixed Safari layout bug"]
        );
        assert_eq!(
            description_items("- just one dash line"),
            ["- just one dash line"]
        );
        assert_eq!(description_items("- a\nplain line"), ["- a plain line"]);
    }

    #[test]
    fn names_and_descriptions_are_required() {
        assert_eq!(require_name("  "), Err(ModelError::NameRequired));
        assert_eq!(require_name(" Acme ").unwrap(), "Acme");
        assert_eq!(
            require_description(""),
            Err(ModelError::DescriptionRequired)
        );
    }
}
