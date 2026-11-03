//! The record model (plan P0-4): every record carries an identifier, a
//! revision, provenance and a tombstone flag, and every change appends a
//! history entry with the before and after values. The full state of a
//! record can be rebuilt from its history alone.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::id::{Id, IdGenerator};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RecordError {
    #[error("record is deleted")]
    Deleted,
    #[error("history is empty")]
    EmptyHistory,
    #[error("history is broken at revision {0}")]
    BrokenHistory(u64),
}

impl RecordError {
    pub fn code(&self) -> &'static str {
        match self {
            RecordError::Deleted => "record_deleted",
            RecordError::EmptyHistory => "history_empty",
            RecordError::BrokenHistory(_) => "history_broken",
        }
    }
}

/// Who made a change, and from which device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub author: String,
    pub device: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Create,
    Revise,
    Delete,
}

impl Op {
    pub fn as_str(self) -> &'static str {
        match self {
            Op::Create => "create",
            Op::Revise => "revise",
            Op::Delete => "delete",
        }
    }

    pub fn parse(text: &str) -> Option<Op> {
        match text {
            "create" => Some(Op::Create),
            "revise" => Some(Op::Revise),
            "delete" => Some(Op::Delete),
            _ => None,
        }
    }
}

/// A record of any kind (client, project, time entry, ...). The kind-specific
/// fields live in `payload`; the envelope is what synchronization needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub id: Id,
    pub kind: String,
    pub revision: u64,
    pub deleted: bool,
    pub created_by: Actor,
    pub updated_by: Actor,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub payload: Value,
}

/// One append-only history entry. `before` is absent for a create and `after`
/// is absent for a delete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub record_id: Id,
    pub kind: String,
    pub revision: u64,
    pub op: Op,
    pub actor: Actor,
    pub at: DateTime<Utc>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

impl Record {
    pub fn create(
        ids: &IdGenerator,
        kind: &str,
        payload: Value,
        actor: Actor,
        now: DateTime<Utc>,
    ) -> (Record, HistoryEntry) {
        let record = Record {
            id: ids.next_id(),
            kind: kind.to_owned(),
            revision: 1,
            deleted: false,
            created_by: actor.clone(),
            updated_by: actor.clone(),
            created_at: now,
            updated_at: now,
            payload: payload.clone(),
        };
        let entry = HistoryEntry {
            record_id: record.id.clone(),
            kind: record.kind.clone(),
            revision: 1,
            op: Op::Create,
            actor,
            at: now,
            before: None,
            after: Some(payload),
        };
        (record, entry)
    }

    pub fn revise(
        &self,
        payload: Value,
        actor: Actor,
        now: DateTime<Utc>,
    ) -> Result<(Record, HistoryEntry), RecordError> {
        if self.deleted {
            return Err(RecordError::Deleted);
        }
        let next = Record {
            revision: self.revision + 1,
            updated_by: actor.clone(),
            updated_at: now,
            payload: payload.clone(),
            ..self.clone()
        };
        let entry = HistoryEntry {
            record_id: self.id.clone(),
            kind: self.kind.clone(),
            revision: next.revision,
            op: Op::Revise,
            actor,
            at: now,
            before: Some(self.payload.clone()),
            after: Some(payload),
        };
        Ok((next, entry))
    }

    /// Deletion is a tombstone: the record stays, marked deleted, and the last
    /// payload is kept in the history.
    pub fn delete(
        &self,
        actor: Actor,
        now: DateTime<Utc>,
    ) -> Result<(Record, HistoryEntry), RecordError> {
        if self.deleted {
            return Err(RecordError::Deleted);
        }
        let next = Record {
            revision: self.revision + 1,
            deleted: true,
            updated_by: actor.clone(),
            updated_at: now,
            ..self.clone()
        };
        let entry = HistoryEntry {
            record_id: self.id.clone(),
            kind: self.kind.clone(),
            revision: next.revision,
            op: Op::Delete,
            actor,
            at: now,
            before: Some(self.payload.clone()),
            after: None,
        };
        Ok((next, entry))
    }
}

/// Rebuilds a record's state at `up_to` (or at its latest revision) from its
/// history alone, verifying that every entry continues the previous one.
pub fn reconstruct(history: &[HistoryEntry], up_to: Option<u64>) -> Result<Record, RecordError> {
    let mut state: Option<Record> = None;
    for entry in history {
        if up_to.is_some_and(|limit| entry.revision > limit) {
            break;
        }
        state = Some(match (state, entry.op) {
            (None, Op::Create) if entry.revision == 1 && entry.before.is_none() => Record {
                id: entry.record_id.clone(),
                kind: entry.kind.clone(),
                revision: 1,
                deleted: false,
                created_by: entry.actor.clone(),
                updated_by: entry.actor.clone(),
                created_at: entry.at,
                updated_at: entry.at,
                payload: entry.after.clone().ok_or(RecordError::BrokenHistory(1))?,
            },
            (Some(prev), Op::Revise)
                if !prev.deleted
                    && entry.revision == prev.revision + 1
                    && entry.before.as_ref() == Some(&prev.payload) =>
            {
                Record {
                    revision: entry.revision,
                    updated_by: entry.actor.clone(),
                    updated_at: entry.at,
                    payload: entry
                        .after
                        .clone()
                        .ok_or(RecordError::BrokenHistory(entry.revision))?,
                    ..prev
                }
            }
            (Some(prev), Op::Delete)
                if !prev.deleted
                    && entry.revision == prev.revision + 1
                    && entry.before.as_ref() == Some(&prev.payload) =>
            {
                Record {
                    revision: entry.revision,
                    deleted: true,
                    updated_by: entry.actor.clone(),
                    updated_at: entry.at,
                    ..prev
                }
            }
            _ => return Err(RecordError::BrokenHistory(entry.revision)),
        });
    }
    state.ok_or(RecordError::EmptyHistory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Origin;
    use serde_json::json;

    fn actor() -> Actor {
        Actor {
            author: "baden".into(),
            device: "mac-1".into(),
        }
    }

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn create_revise_delete_and_reconstruct() {
        let ids = IdGenerator::new(Origin::new("test").unwrap());
        let (r1, h1) = Record::create(
            &ids,
            "client",
            json!({"name": "Acme"}),
            actor(),
            at("2026-11-03T09:00:00Z"),
        );
        let (r2, h2) = r1
            .revise(
                json!({"name": "Acme GmbH"}),
                actor(),
                at("2026-11-03T10:00:00Z"),
            )
            .unwrap();
        let (r3, h3) = r2.delete(actor(), at("2026-11-03T11:00:00Z")).unwrap();
        let history = vec![h1, h2, h3];

        assert_eq!(r3.revision, 3);
        assert!(r3.deleted);
        assert_eq!(reconstruct(&history, None).unwrap(), r3);
        assert_eq!(reconstruct(&history, Some(2)).unwrap(), r2);
        assert_eq!(reconstruct(&history, Some(1)).unwrap(), r1);
        assert_eq!(history[1].before, Some(json!({"name": "Acme"})));
        assert_eq!(history[1].after, Some(json!({"name": "Acme GmbH"})));
    }

    #[test]
    fn deleted_records_cannot_change() {
        let ids = IdGenerator::new(Origin::random());
        let (r, _) = Record::create(&ids, "tag", json!({}), actor(), at("2026-11-03T09:00:00Z"));
        let (d, _) = r.delete(actor(), at("2026-11-03T09:01:00Z")).unwrap();
        assert_eq!(
            d.revise(json!({}), actor(), at("2026-11-03T09:02:00Z"))
                .unwrap_err(),
            RecordError::Deleted
        );
        assert_eq!(
            d.delete(actor(), at("2026-11-03T09:02:00Z")).unwrap_err(),
            RecordError::Deleted
        );
    }

    #[test]
    fn broken_history_is_detected() {
        let ids = IdGenerator::new(Origin::random());
        let (r1, h1) = Record::create(
            &ids,
            "tag",
            json!({"a": 1}),
            actor(),
            at("2026-11-03T09:00:00Z"),
        );
        let (_, mut h2) = r1
            .revise(json!({"a": 2}), actor(), at("2026-11-03T09:01:00Z"))
            .unwrap();
        h2.before = Some(json!({"a": 99}));
        assert_eq!(
            reconstruct(&[h1.clone(), h2], None).unwrap_err(),
            RecordError::BrokenHistory(2)
        );
        assert_eq!(
            reconstruct(&[], None).unwrap_err(),
            RecordError::EmptyHistory
        );
        // history that skips a revision
        let (_, h3) = r1
            .revise(json!({"a": 3}), actor(), at("2026-11-03T09:02:00Z"))
            .unwrap();
        let mut skipped = h3;
        skipped.revision = 5;
        assert!(reconstruct(&[h1, skipped], None).is_err());
    }
}
