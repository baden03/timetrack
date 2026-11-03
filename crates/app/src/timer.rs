//! The live timer (concept §9): one per person, stored so it survives a quit
//! or a crash. Stopping turns it into an ordinary interval entry (split at
//! midnight) in the same transaction that removes the timer.

use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use timetrack_core::id::Id;
use timetrack_core::model::{
    compose_description, normalize_tags, TimeEntry, Timer, KIND_ENTRY, KIND_TIMER,
};
use timetrack_core::record::Record;
use timetrack_core::time::{local_date, local_to_utc, TimeError};
use timetrack_storage::Store;

use crate::entries::{double_option, kind_of_interval, SavedEntry};
use crate::error::{AppError, Result};
use crate::service::{payload, App};

#[derive(Debug, Clone, Deserialize)]
pub struct StartTimer {
    pub project_id: Id,
    #[serde(default)]
    pub category_id: Option<Id>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub billable: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StopTimer {
    /// The last piece of the description, typed at the stop. Together with the
    /// notes added while the timer ran it must give at least one line.
    #[serde(default)]
    pub description: String,
    /// The real stop time as `HH:MM` on the start's date (next day if it is
    /// not later than the start). Without it the timer stops now.
    pub end_time: Option<String>,
    pub tags: Option<Vec<String>>,
    /// Absent: keep the timer's category. `null`: no category.
    #[serde(default, deserialize_with = "double_option")]
    pub category_id: Option<Option<Id>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerState {
    pub timer: Option<Record>,
    /// Started on an earlier day: the person is asked for the real stop time
    /// or to discard it (the only notification in the first milestone).
    pub stale: bool,
    /// The category last used on the timer's project, proposed at the stop
    /// when the timer itself has none.
    pub suggested_category_id: Option<Id>,
}

fn to_minute(t: DateTime<Utc>) -> DateTime<Utc> {
    let secs = t.timestamp();
    Utc.timestamp_opt(secs - secs.rem_euclid(60), 0)
        .single()
        .expect("valid timestamp")
}

impl<S: Store> App<S> {
    fn current_timer(&self) -> Result<Option<Record>> {
        Ok(self.store.list(KIND_TIMER, false)?.into_iter().next())
    }

    pub fn timer_state(&self) -> Result<TimerState> {
        let timer = self.current_timer()?;
        let stale = match &timer {
            Some(rec) => {
                let t: Timer = payload(rec)?;
                let zone: Tz = t.timezone.parse().map_err(|_| AppError::UnknownTimeZone)?;
                local_date(t.start_at, zone) < local_date(self.now(), zone)
            }
            None => false,
        };
        let suggested_category_id = match &timer {
            Some(rec) => {
                let t: Timer = payload(rec)?;
                self.last_used_category(&t.project_id)?
            }
            None => None,
        };
        Ok(TimerState {
            timer,
            stale,
            suggested_category_id,
        })
    }

    pub fn start_timer(&mut self, new: StartTimer) -> Result<Record> {
        if self.current_timer()?.is_some() {
            return Err(AppError::TimerRunning);
        }
        self.check_refs(&new.project_id, new.category_id.as_ref())?;
        let timer = Timer {
            project_id: new.project_id,
            category_id: new.category_id,
            description: new.description.trim().to_owned(),
            notes: vec![],
            billable: new.billable,
            tags: normalize_tags(&new.tags)?,
            start_at: to_minute(self.now()),
            exact_start_at: Some(self.now()),
            timezone: self.settings_value()?.timezone,
        };
        self.create(KIND_TIMER, &timer)
    }

    /// Starts a timer on the project and category of the most recent entry
    /// (what the global shortcut does).
    pub fn quick_start_timer(&mut self) -> Result<Record> {
        let newest = self
            .store
            .list_by_date(KIND_ENTRY, None, None)?
            .into_iter()
            .max_by(|a, b| a.id.cmp(&b.id))
            .ok_or(AppError::NothingToRepeat)?;
        let e: TimeEntry = payload(&newest)?;
        let category_id = self.last_used_category(&e.project_id)?;
        self.start_timer(StartTimer {
            project_id: e.project_id,
            category_id,
            description: String::new(),
            tags: vec![],
            billable: Some(e.billable),
        })
    }

    /// Saves the description and tags typed while the timer runs.
    pub fn update_timer(
        &mut self,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> Result<Record> {
        let rec = self.current_timer()?.ok_or(AppError::NoTimer)?;
        let mut t: Timer = payload(&rec)?;
        if let Some(d) = description {
            t.description = d.trim().to_owned();
        }
        if let Some(tags) = tags {
            t.tags = normalize_tags(&tags)?;
        }
        self.revise(&rec, &t)
    }

    /// Adds a line while the timer runs (a bullet point of the entry later).
    pub fn add_timer_note(&mut self, text: &str) -> Result<Record> {
        let rec = self.current_timer()?.ok_or(AppError::NoTimer)?;
        let mut t: Timer = payload(&rec)?;
        let note = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if note.is_empty() {
            return Err(timetrack_core::model::ModelError::DescriptionRequired.into());
        }
        t.notes.push(note);
        // What was typed in the note field has been entered as a note.
        t.description.clear();
        self.revise(&rec, &t)
    }

    /// Removes a running note by its position.
    pub fn remove_timer_note(&mut self, index: usize) -> Result<Record> {
        let rec = self.current_timer()?.ok_or(AppError::NoTimer)?;
        let mut t: Timer = payload(&rec)?;
        if index >= t.notes.len() {
            return Err(AppError::NotFound);
        }
        t.notes.remove(index);
        self.revise(&rec, &t)
    }

    pub fn stop_timer(&mut self, stop: StopTimer) -> Result<SavedEntry> {
        let rec = self.current_timer()?.ok_or(AppError::NoTimer)?;
        let t: Timer = payload(&rec)?;
        let mut pieces_of_text = t.notes.clone();
        pieces_of_text.push(stop.description.clone());
        let description = compose_description(&pieces_of_text)?;
        let zone: Tz = t.timezone.parse().map_err(|_| AppError::UnknownTimeZone)?;
        let category_id = stop.category_id.clone().unwrap_or(t.category_id.clone());
        let project = self.check_refs(&t.project_id, category_id.as_ref())?;

        let end_at = match &stop.end_time {
            None => to_minute(self.now()),
            Some(text) => {
                let time = NaiveTime::parse_from_str(text, "%H:%M")
                    .map_err(|_| AppError::InvalidDateTime)?;
                let start_local = t.start_at.with_timezone(&zone).naive_local();
                let mut end = start_local.date().and_time(time);
                if end.time() == start_local.time() {
                    return Err(TimeError::EndNotAfterStart.into());
                }
                if end <= start_local {
                    end += Duration::days(1);
                }
                local_to_utc(end, zone)?
            }
        };
        if stop.end_time.is_none() && end_at <= t.start_at {
            // Entries are whole minutes; the person can keep going or discard.
            return Err(AppError::TimerTooShort);
        }
        let specs = self.specs_from_instants(t.start_at, end_at, zone)?;
        let overlaps = self.find_overlaps(&specs, None)?;

        let base = TimeEntry {
            project_id: t.project_id,
            category_id,
            description,
            billable: t.billable.unwrap_or(project.billable_default),
            tags: match stop.tags {
                Some(tags) => normalize_tags(&tags)?,
                None => t.tags,
            },
            kind: kind_of_interval(),
            work_date: specs[0].work_date,
            timezone: t.timezone,
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
        let mut changes = self.create_pieces(base, &specs);
        let pieces: Vec<Record> = changes.iter().map(|(r, _)| r.clone()).collect();
        changes.push(rec.delete(self.actor.clone(), self.now())?);
        self.store.put_many(&changes)?;
        Ok(SavedEntry { pieces, overlaps })
    }

    pub fn discard_timer(&mut self) -> Result<()> {
        let rec = self.current_timer()?.ok_or(AppError::NoTimer)?;
        let change = rec.delete(self.actor.clone(), self.now())?;
        self.store.put(&change.0, &change.1)?;
        Ok(())
    }
}
