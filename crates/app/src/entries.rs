//! Time entries: creating, editing and deleting, including the midnight
//! split performed by this (the author's) client.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use timetrack_core::id::Id;
use timetrack_core::model::{
    normalize_tags, require_description, Category, EntryKind, Project, Status, TimeEntry,
    KIND_CATEGORY, KIND_ENTRY, KIND_PROJECT,
};
use timetrack_core::record::Record;
use timetrack_core::time::{
    check_split_group, intervals_overlap, local_date, local_to_utc, minutes_between,
    split_at_midnight, validate_duration, validate_interval, TimeError,
};
use timetrack_storage::Store;

use crate::error::{AppError, Result};
use crate::service::{payload, to_value, App};

/// How the time of an entry is given. Dates are `YYYY-MM-DD`, times `HH:MM`
/// in the entry's zone. An end earlier than the start means the next day.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Timing {
    Interval {
        date: String,
        start: String,
        end: String,
    },
    Duration {
        date: String,
        minutes: i64,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewEntry {
    pub project_id: Id,
    #[serde(default)]
    pub category_id: Option<Id>,
    pub description: String,
    /// Defaults from the project.
    pub billable: Option<bool>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub timing: Timing,
    /// IANA zone; defaults to the zone in the settings.
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EntryPatch {
    pub project_id: Option<Id>,
    /// Absent: unchanged. `null`: remove the category.
    #[serde(default, deserialize_with = "double_option")]
    pub category_id: Option<Option<Id>>,
    pub description: Option<String>,
    pub billable: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SavedEntry {
    /// The stored pieces, in order. More than one means a midnight split.
    pub pieces: Vec<Record>,
    /// Groups of the same person's entries that overlap this one. A warning,
    /// never a reason to refuse the save (concept §8.4).
    pub overlaps: Vec<String>,
}

/// Defaults proposed for the entry form (concept §8.6).
#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    /// End of the previous interval that day, as `HH:MM`.
    pub start: Option<String>,
    /// Category last used on the project.
    pub category_id: Option<Id>,
}

pub(crate) struct Spec {
    pub(crate) work_date: NaiveDate,
    pub(crate) minutes: i64,
    pub(crate) start_at: Option<DateTime<Utc>>,
    pub(crate) end_at: Option<DateTime<Utc>>,
    pub(crate) start_offset: Option<i32>,
    pub(crate) end_offset: Option<i32>,
    pub(crate) span: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

fn parse_date(text: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| AppError::InvalidDateTime)
}

fn parse_time(text: &str) -> Result<NaiveTime> {
    NaiveTime::parse_from_str(text, "%H:%M").map_err(|_| AppError::InvalidDateTime)
}

fn parse_zone(name: &str) -> Result<Tz> {
    name.parse::<Tz>().map_err(|_| AppError::UnknownTimeZone)
}

impl<S: Store> App<S> {
    pub fn create_entry(&mut self, new: NewEntry) -> Result<SavedEntry> {
        let description = require_description(&new.description)?;
        let tags = normalize_tags(&new.tags)?;
        let project = self.check_refs(&new.project_id, new.category_id.as_ref())?;
        let zone_name = match new.timezone {
            Some(z) => z,
            None => self.settings_value()?.timezone,
        };
        let zone = parse_zone(&zone_name)?;
        let specs = self.resolve(&new.timing, zone)?;
        let overlaps = self.find_overlaps(&specs, None)?;

        let base = TimeEntry {
            project_id: new.project_id,
            category_id: new.category_id,
            description,
            billable: new.billable.unwrap_or(project.billable_default),
            tags,
            kind: kind_of(&new.timing),
            work_date: specs[0].work_date,
            timezone: zone_name,
            minutes: 0,
            start_at: None,
            end_at: None,
            start_offset: None,
            end_offset: None,
            group_id: String::new(),
            piece_index: 0,
            piece_count: 0,
            span_start_at: None,
            span_end_at: None,
        };
        let changes = self.create_pieces(base, &specs);
        self.store.put_many(&changes)?;
        Ok(SavedEntry {
            pieces: changes.into_iter().map(|(r, _)| r).collect(),
            overlaps,
        })
    }

    /// Edits all pieces of an entry as one change. Editing only the text
    /// fields never recalculates the split; editing the timing does, on this
    /// device, in the entry's recorded zone (concept §8.5).
    pub fn update_entry(&mut self, group: &str, patch: EntryPatch) -> Result<SavedEntry> {
        let old = self.group(group)?;
        let first: TimeEntry = payload(&old[0])?;
        let mut base = first.clone();

        if patch.project_id.is_some() || patch.category_id.is_some() {
            let project_id = patch.project_id.clone().unwrap_or(first.project_id.clone());
            let category_id = patch
                .category_id
                .clone()
                .unwrap_or(first.category_id.clone());
            self.check_refs(&project_id, category_id.as_ref())?;
            base.project_id = project_id;
            base.category_id = category_id;
        }
        if let Some(d) = &patch.description {
            base.description = require_description(d)?;
        }
        if let Some(b) = patch.billable {
            base.billable = b;
        }
        if let Some(t) = &patch.tags {
            base.tags = normalize_tags(t)?;
        }

        let mut changes = Vec::new();
        let mut overlaps = Vec::new();
        match &patch.timing {
            None => {
                for rec in &old {
                    let mut e: TimeEntry = payload(rec)?;
                    e.project_id = base.project_id.clone();
                    e.category_id = base.category_id.clone();
                    e.description = base.description.clone();
                    e.billable = base.billable;
                    e.tags = base.tags.clone();
                    changes.push(rec.revise(to_value(&e), self.actor.clone(), self.now())?);
                }
            }
            Some(timing) => {
                let zone = parse_zone(&first.timezone)?;
                let specs = self.resolve(timing, zone)?;
                overlaps = self.find_overlaps(&specs, Some(group))?;
                base.kind = kind_of(timing);
                base.work_date = specs[0].work_date;
                for (i, spec) in specs.iter().enumerate() {
                    let entry = piece(&base, spec, i, specs.len());
                    changes.push(match old.get(i) {
                        Some(rec) => {
                            rec.revise(to_value(&entry), self.actor.clone(), self.now())?
                        }
                        None => Record::create(
                            &self.ids,
                            KIND_ENTRY,
                            to_value(&entry),
                            self.actor.clone(),
                            self.now(),
                        ),
                    });
                }
                // Pieces that no longer exist (the entry now ends before
                // midnight) are tombstoned.
                for rec in old.iter().skip(specs.len()) {
                    changes.push(rec.delete(self.actor.clone(), self.now())?);
                }
            }
        }
        self.store.put_many(&changes)?;
        let pieces = changes
            .into_iter()
            .map(|(r, _)| r)
            .filter(|r| !r.deleted)
            .collect();
        Ok(SavedEntry { pieces, overlaps })
    }

    /// Deletes every piece of an entry (tombstones; the history keeps them).
    pub fn delete_entry(&mut self, group: &str) -> Result<()> {
        let changes = self
            .group(group)?
            .iter()
            .map(|r| r.delete(self.actor.clone(), self.now()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        self.store.put_many(&changes)?;
        Ok(())
    }

    /// Entries whose date lies in `[from, to]` (ISO dates), oldest first.
    pub fn list_entries(&self, from: Option<&str>, to: Option<&str>) -> Result<Vec<Record>> {
        Ok(self.store.list_by_date(KIND_ENTRY, from, to)?)
    }

    pub fn entry_group(&self, group: &str) -> Result<Vec<Record>> {
        self.group(group)
    }

    pub fn suggest(&self, project_id: &Id, date: &str) -> Result<Suggestion> {
        let zone = parse_zone(&self.settings_value()?.timezone)?;
        let day = parse_date(date)?;
        let iso = day.to_string();
        let mut latest_end: Option<DateTime<Utc>> = None;
        for rec in self
            .store
            .list_by_date(KIND_ENTRY, Some(&iso), Some(&iso))?
        {
            let e: TimeEntry = payload(&rec)?;
            if let Some(end) = e.end_at {
                // A piece ending at the next local midnight leaves no room.
                if local_date(end, zone) == day && latest_end.is_none_or(|l| end > l) {
                    latest_end = Some(end);
                }
            }
        }
        let start = latest_end.map(|t| t.with_timezone(&zone).format("%H:%M").to_string());

        let mut category_id = None;
        let mut newest: Option<Id> = None;
        for rec in self.store.list_by_date(KIND_ENTRY, None, None)? {
            let e: TimeEntry = payload(&rec)?;
            if e.project_id == *project_id && newest.as_ref().is_none_or(|n| rec.id > *n) {
                newest = Some(rec.id.clone());
                category_id = e.category_id;
            }
        }
        Ok(Suggestion { start, category_id })
    }

    // ---- internals ------------------------------------------------------

    /// Builds the new records for the pieces of one entry without storing them.
    pub(crate) fn create_pieces(
        &self,
        mut base: TimeEntry,
        specs: &[Spec],
    ) -> Vec<(Record, timetrack_core::record::HistoryEntry)> {
        base.group_id = self.ids.next_id().to_string();
        specs
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                Record::create(
                    &self.ids,
                    KIND_ENTRY,
                    to_value(&piece(&base, spec, i, specs.len())),
                    self.actor.clone(),
                    self.now(),
                )
            })
            .collect()
    }

    /// Splits an interval given as instants, validating it like any entry.
    pub(crate) fn specs_from_instants(
        &self,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        zone: Tz,
    ) -> Result<Vec<Spec>> {
        validate_interval(start_at, end_at, self.now())?;
        let pieces = split_at_midnight(start_at, end_at, zone)?;
        check_split_group(&pieces)?;
        Ok(pieces
            .into_iter()
            .map(|p| Spec {
                work_date: p.work_date,
                minutes: p.minutes,
                start_at: Some(p.start_at),
                end_at: Some(p.end_at),
                start_offset: Some(p.start_offset),
                end_offset: Some(p.end_offset),
                span: Some((p.span_start_at, p.span_end_at)),
            })
            .collect())
    }

    fn group(&self, group: &str) -> Result<Vec<Record>> {
        let mut pieces = self.store.list_by_group(KIND_ENTRY, group)?;
        if pieces.is_empty() {
            return Err(AppError::NotFound);
        }
        let mut keyed = Vec::new();
        for rec in pieces.drain(..) {
            let e: TimeEntry = payload(&rec)?;
            keyed.push((e.piece_index, rec));
        }
        keyed.sort_by_key(|(i, _)| *i);
        Ok(keyed.into_iter().map(|(_, r)| r).collect())
    }

    /// The project must be active; a category, if given, must be active and
    /// enabled for the project.
    pub(crate) fn check_refs(&self, project_id: &Id, category_id: Option<&Id>) -> Result<Project> {
        let project: Project = payload(&self.load(project_id, KIND_PROJECT)?)?;
        if project.status == Status::Archived {
            return Err(AppError::ProjectArchived);
        }
        if let Some(category_id) = category_id {
            let category: Category = payload(&self.load(category_id, KIND_CATEGORY)?)?;
            if category.status == Status::Archived {
                return Err(AppError::CategoryArchived);
            }
            if !project.category_ids.contains(category_id) {
                return Err(AppError::CategoryNotEnabled);
            }
        }
        Ok(project)
    }

    fn resolve(&self, timing: &Timing, zone: Tz) -> Result<Vec<Spec>> {
        let now = self.now();
        match timing {
            Timing::Interval { date, start, end } => {
                let day = parse_date(date)?;
                let start_t = parse_time(start)?;
                let end_t = parse_time(end)?;
                if end_t == start_t {
                    return Err(TimeError::EndNotAfterStart.into());
                }
                let end_day = if end_t > start_t {
                    day
                } else {
                    day + Duration::days(1)
                };
                let start_at = local_to_utc(day.and_time(start_t), zone)?;
                let end_at = local_to_utc(end_day.and_time(end_t), zone)?;
                self.specs_from_instants(start_at, end_at, zone)
            }
            Timing::Duration { date, minutes } => {
                let day = parse_date(date)?;
                validate_duration(*minutes)?;
                if day > local_date(now, zone) {
                    return Err(TimeError::InFuture.into());
                }
                Ok(vec![Spec {
                    work_date: day,
                    minutes: *minutes,
                    start_at: None,
                    end_at: None,
                    start_offset: None,
                    end_offset: None,
                    span: None,
                }])
            }
        }
    }

    /// Groups of other entries whose intervals overlap these pieces, judged
    /// on instants. Duration-only entries never take part.
    pub(crate) fn find_overlaps(
        &self,
        specs: &[Spec],
        own_group: Option<&str>,
    ) -> Result<Vec<String>> {
        let (Some(first), Some(last)) = (specs.first(), specs.last()) else {
            return Ok(vec![]);
        };
        if first.start_at.is_none() {
            return Ok(vec![]);
        }
        let from = (first.work_date - Duration::days(1)).to_string();
        let to = (last.work_date + Duration::days(1)).to_string();
        let mut found: Vec<String> = Vec::new();
        for rec in self
            .store
            .list_by_date(KIND_ENTRY, Some(&from), Some(&to))?
        {
            let e: TimeEntry = payload(&rec)?;
            if own_group == Some(e.group_id.as_str()) || found.contains(&e.group_id) {
                continue;
            }
            let (Some(s), Some(t)) = (e.start_at, e.end_at) else {
                continue;
            };
            let clash = specs.iter().any(|sp| match (sp.start_at, sp.end_at) {
                (Some(a), Some(b)) => intervals_overlap((a, b), (s, t)),
                _ => false,
            });
            if clash {
                found.push(e.group_id);
            }
        }
        Ok(found)
    }
}

pub(crate) fn kind_of(timing: &Timing) -> EntryKind {
    match timing {
        Timing::Interval { .. } => EntryKind::Interval,
        Timing::Duration { .. } => EntryKind::Duration,
    }
}

fn piece(base: &TimeEntry, spec: &Spec, index: usize, count: usize) -> TimeEntry {
    debug_assert!(spec.start_at.is_none_or(|s| spec
        .end_at
        .is_some_and(|e| minutes_between(s, e) == spec.minutes)));
    TimeEntry {
        work_date: spec.work_date,
        minutes: spec.minutes,
        start_at: spec.start_at,
        end_at: spec.end_at,
        start_offset: spec.start_offset,
        end_offset: spec.end_offset,
        piece_index: index as u32,
        piece_count: count as u32,
        span_start_at: spec.span.map(|s| s.0),
        span_end_at: spec.span.map(|s| s.1),
        ..base.clone()
    }
}

pub(crate) fn kind_of_interval() -> EntryKind {
    EntryKind::Interval
}

/// Distinguishes a missing field (`None`) from an explicit `null` (`Some(None)`).
pub(crate) fn double_option<'de, D, T>(d: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Some(Option::deserialize(d)?))
}
