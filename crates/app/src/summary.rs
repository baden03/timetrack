use chrono::{Datelike, Duration, NaiveDate};
use serde::Serialize;
use timetrack_core::i18n::Catalog;
use timetrack_core::model::{
    Category, Client, Project, TimeEntry, KIND_CATEGORY, KIND_CLIENT, KIND_ENTRY, KIND_PROJECT,
};
use timetrack_core::summary::{build, Summary, SummaryEntry};
use timetrack_storage::Store;

use crate::error::{AppError, Result};
use crate::service::{payload, App};

#[derive(Debug, Clone, Serialize)]
pub struct MonthlySummary {
    pub summary: Summary,
    /// The copyable plain text (docs/summary-format.md).
    pub text: String,
}

impl<S: Store> App<S> {
    /// The billable time of one calendar month (`YYYY-MM`). Each stored piece
    /// counts in the month of its own local date.
    pub fn monthly_summary(&self, month: &str) -> Result<MonthlySummary> {
        let first = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
            .map_err(|_| AppError::InvalidMonth)?;
        let next = if first.month() == 12 {
            NaiveDate::from_ymd_opt(first.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(first.year(), first.month() + 1, 1)
        }
        .ok_or(AppError::InvalidMonth)?;
        let last = next - Duration::days(1);

        let mut pieces: Vec<(TimeEntry, String)> = Vec::new();
        for rec in self.store.list_by_date(
            KIND_ENTRY,
            Some(&first.to_string()),
            Some(&last.to_string()),
        )? {
            let e: TimeEntry = payload(&rec)?;
            if e.billable {
                pieces.push((e, rec.id.to_string()));
            }
        }
        pieces.sort_by(|a, b| {
            (a.0.work_date, a.0.start_at, &a.1).cmp(&(b.0.work_date, b.0.start_at, &b.1))
        });

        let clients: Vec<(timetrack_core::id::Id, Client)> = self.named(KIND_CLIENT)?;
        let projects: Vec<(timetrack_core::id::Id, Project)> = self.named(KIND_PROJECT)?;
        let categories: Vec<(timetrack_core::id::Id, Category)> = self.named(KIND_CATEGORY)?;
        let mut entries = Vec::new();
        for (e, _) in pieces {
            let project = projects
                .iter()
                .find(|p| p.0 == e.project_id)
                .ok_or(AppError::NotFound)?;
            let client = clients
                .iter()
                .find(|c| c.0 == project.1.client_id)
                .ok_or(AppError::NotFound)?;
            let category = match &e.category_id {
                Some(id) => Some(
                    categories
                        .iter()
                        .find(|c| c.0 == *id)
                        .ok_or(AppError::NotFound)?,
                ),
                None => None,
            };
            entries.push(SummaryEntry {
                client_id: client.0.clone(),
                client_name: client.1.name.clone(),
                project_id: project.0.clone(),
                project_name: project.1.name.clone(),
                category_id: category.map(|c| c.0.clone()),
                category_name: category.map(|c| c.1.name.clone()).unwrap_or_default(),
                description: e.description,
                minutes: e.minutes,
            });
        }
        let summary = build(&entries);
        let text = summary.to_text(&Catalog::english(), first.year(), first.month());
        Ok(MonthlySummary { summary, text })
    }

    fn named<T: serde::de::DeserializeOwned>(
        &self,
        kind: &str,
    ) -> Result<Vec<(timetrack_core::id::Id, T)>> {
        self.store
            .list(kind, false)?
            .into_iter()
            .map(|r| Ok((r.id.clone(), payload(&r)?)))
            .collect()
    }
}
