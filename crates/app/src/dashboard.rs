//! The dashboard (concept §14.1) and the overview of one project.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use timetrack_core::id::Id;
use timetrack_core::model::{
    Client, Project, Status, TimeEntry, Timer, KIND_CLIENT, KIND_ENTRY, KIND_PROJECT, KIND_TIMER,
};
use timetrack_core::record::Record;
use timetrack_core::time::local_date;
use timetrack_storage::Store;

use crate::error::{AppError, Result};
use crate::service::{payload, App};

#[derive(Debug, Clone, Serialize)]
pub struct ProjectCard {
    pub project: Record,
    pub client_name: String,
    /// When time was last recorded on the project (the project's creation if
    /// none), or now while its timer runs. Cards are ordered by this.
    pub last_activity: DateTime<Utc>,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectOverview {
    pub today_minutes: i64,
    pub week_minutes: i64,
    pub month_minutes: i64,
    pub total_minutes: i64,
}

impl<S: Store> App<S> {
    /// Active projects of active clients, most recently used first.
    pub fn dashboard(&self) -> Result<Vec<ProjectCard>> {
        let clients: Vec<(Id, Client)> = self
            .store
            .list(KIND_CLIENT, false)?
            .into_iter()
            .map(|r| Ok((r.id.clone(), payload::<Client>(&r)?)))
            .collect::<Result<_>>()?;

        let mut activity: std::collections::HashMap<Id, DateTime<Utc>> = Default::default();
        for rec in self.store.list_by_date(KIND_ENTRY, None, None)? {
            let e: TimeEntry = payload(&rec)?;
            let seen = activity.entry(e.project_id).or_insert(rec.updated_at);
            if rec.updated_at > *seen {
                *seen = rec.updated_at;
            }
        }
        let running: Option<Id> = match self.store.list(KIND_TIMER, false)?.first() {
            Some(rec) => Some(payload::<Timer>(rec)?.project_id),
            None => None,
        };

        let mut cards = Vec::new();
        for rec in self.store.list(KIND_PROJECT, false)? {
            let p: Project = payload(&rec)?;
            let Some((_, client)) = clients.iter().find(|c| c.0 == p.client_id) else {
                continue;
            };
            if p.status == Status::Archived || client.status == Status::Archived {
                continue;
            }
            let is_running = running.as_ref() == Some(&rec.id);
            let last_activity = if is_running {
                self.now()
            } else {
                activity
                    .get(&rec.id)
                    .copied()
                    .map_or(rec.created_at, |a| a.max(rec.created_at))
            };
            cards.push(ProjectCard {
                client_name: client.name.clone(),
                last_activity,
                running: is_running,
                project: rec,
            });
        }
        cards.sort_by(|a, b| {
            b.last_activity
                .cmp(&a.last_activity)
                .then(b.project.id.cmp(&a.project.id))
        });
        Ok(cards)
    }

    /// Totals for one project: today, this week (Monday to Sunday), this
    /// month and all time, by each piece's own date in the settings zone.
    pub fn project_overview(&self, project_id: &Id) -> Result<ProjectOverview> {
        self.store.get(project_id)?.ok_or(AppError::NotFound)?;
        let zone: Tz = self
            .settings_value()?
            .timezone
            .parse()
            .map_err(|_| AppError::UnknownTimeZone)?;
        let today: NaiveDate = local_date(self.now(), zone);
        let week_start = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
        let week_end = week_start + Duration::days(6);

        let mut o = ProjectOverview {
            today_minutes: 0,
            week_minutes: 0,
            month_minutes: 0,
            total_minutes: 0,
        };
        for rec in self.store.list_by_date(KIND_ENTRY, None, None)? {
            let e: TimeEntry = payload(&rec)?;
            if e.project_id != *project_id {
                continue;
            }
            o.total_minutes += e.minutes;
            if e.work_date.year() == today.year() && e.work_date.month() == today.month() {
                o.month_minutes += e.minutes;
            }
            if e.work_date >= week_start && e.work_date <= week_end {
                o.week_minutes += e.minutes;
            }
            if e.work_date == today {
                o.today_minutes += e.minutes;
            }
        }
        Ok(o)
    }
}
