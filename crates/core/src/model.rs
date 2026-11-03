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
    pub category_id: Id,
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
    fn names_and_descriptions_are_required() {
        assert_eq!(require_name("  "), Err(ModelError::NameRequired));
        assert_eq!(require_name(" Acme ").unwrap(), "Acme");
        assert_eq!(
            require_description(""),
            Err(ModelError::DescriptionRequired)
        );
    }
}
