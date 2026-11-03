//! Identifiers (concept §3.5): created on the client, never reused, roughly
//! ordered by creation time and origin-aware.
//!
//! Format: `<uuid v7>@<origin>`. The time-ordered UUID comes first so that
//! plain string order is creation order; the origin names the installation
//! (later: the hub) that minted the identifier.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("invalid origin")]
    InvalidOrigin,
    #[error("invalid identifier")]
    InvalidId,
}

impl IdError {
    pub fn code(&self) -> &'static str {
        match self {
            IdError::InvalidOrigin => "invalid_origin",
            IdError::InvalidId => "invalid_id",
        }
    }
}

/// Where an identifier was minted: 1 to 32 characters of `a-z`, `0-9`, `-`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Origin(String);

impl Origin {
    pub fn new(value: &str) -> Result<Self, IdError> {
        let ok = !value.is_empty()
            && value.len() <= 32
            && value
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if ok {
            Ok(Origin(value.to_owned()))
        } else {
            Err(IdError::InvalidOrigin)
        }
    }

    /// A fresh random origin for a new installation.
    pub fn random() -> Self {
        let hex = Uuid::new_v4().simple().to_string();
        Origin(format!("i{}", &hex[..12]))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Origin {
    type Error = IdError;
    fn try_from(value: String) -> Result<Self, IdError> {
        Origin::new(&value)
    }
}

impl From<Origin> for String {
    fn from(value: Origin) -> String {
        value.0
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A record identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Id(String);

impl Id {
    pub fn parse(value: &str) -> Result<Self, IdError> {
        let (uuid, origin) = value.split_once('@').ok_or(IdError::InvalidId)?;
        let parsed = Uuid::parse_str(uuid).map_err(|_| IdError::InvalidId)?;
        if parsed.get_version_num() != 7 || uuid != parsed.hyphenated().to_string() {
            return Err(IdError::InvalidId);
        }
        Origin::new(origin)?;
        Ok(Id(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn origin(&self) -> Origin {
        let (_, origin) = self.0.split_once('@').unwrap_or_default();
        Origin(origin.to_owned())
    }
}

impl TryFrom<String> for Id {
    type Error = IdError;
    fn try_from(value: String) -> Result<Self, IdError> {
        Id::parse(&value)
    }
}

impl From<Id> for String {
    fn from(value: Id) -> String {
        value.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Mints identifiers for one origin. UUID v7 gives a millisecond timestamp
/// plus a monotonic counter inside the process, so identifiers from one
/// generator sort in creation order.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    origin: Origin,
}

impl IdGenerator {
    pub fn new(origin: Origin) -> Self {
        IdGenerator { origin }
    }

    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn next_id(&self) -> Id {
        Id(format!("{}@{}", Uuid::now_v7().hyphenated(), self.origin))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_sort_in_creation_order() {
        let generator = IdGenerator::new(Origin::new("alpha").unwrap());
        let ids: Vec<Id> = (0..20_000).map(|_| generator.next_id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn independent_origins_never_collide() {
        let a = IdGenerator::new(Origin::random());
        let b = IdGenerator::new(Origin::random());
        let mut seen = HashSet::new();
        for _ in 0..10_000 {
            assert!(seen.insert(a.next_id()));
            assert!(seen.insert(b.next_id()));
        }
    }

    #[test]
    fn same_origin_generators_never_collide() {
        let a = IdGenerator::new(Origin::new("same").unwrap());
        let b = IdGenerator::new(Origin::new("same").unwrap());
        let mut seen = HashSet::new();
        for _ in 0..10_000 {
            assert!(seen.insert(a.next_id()));
            assert!(seen.insert(b.next_id()));
        }
    }

    #[test]
    fn round_trips_and_names_origin() {
        let generator = IdGenerator::new(Origin::new("hub-1").unwrap());
        let id = generator.next_id();
        assert_eq!(Id::parse(id.as_str()).unwrap(), id);
        assert_eq!(id.origin().as_str(), "hub-1");
    }

    #[test]
    fn rejects_malformed() {
        assert!(Id::parse("nonsense").is_err());
        assert!(Id::parse("550e8400-e29b-41d4-a716-446655440000@x").is_err()); // v4
        assert!(Origin::new("Upper").is_err());
        assert!(Origin::new("").is_err());
    }
}
