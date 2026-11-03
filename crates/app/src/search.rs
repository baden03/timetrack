//! Search over every entry (concept §14.4), offline, with the total of the
//! matching set.

use serde::{Deserialize, Serialize};
use timetrack_core::id::Id;
use timetrack_core::model::{Project, TimeEntry, KIND_ENTRY};
use timetrack_core::record::Record;
use timetrack_storage::Store;

use crate::error::Result;
use crate::service::{payload, App};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchQuery {
    /// Case-insensitive text in the description.
    pub text: Option<String>,
    pub client_id: Option<Id>,
    pub project_id: Option<Id>,
    pub category_id: Option<Id>,
    /// A tag, ignoring case.
    pub tag: Option<String>,
    pub billable: Option<bool>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// Matching stored pieces, newest date first.
    pub pieces: Vec<Record>,
    pub total_minutes: i64,
}

impl<S: Store> App<S> {
    pub fn search_entries(&self, q: &SearchQuery) -> Result<SearchResult> {
        let text = q
            .text
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_lowercase);
        let tag = q
            .tag
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_lowercase);
        let client_projects: Option<Vec<Id>> = match &q.client_id {
            None => None,
            Some(client) => Some(
                self.store
                    .list(timetrack_core::model::KIND_PROJECT, false)?
                    .into_iter()
                    .filter_map(|r| {
                        payload::<Project>(&r)
                            .ok()
                            .filter(|p| p.client_id == *client)
                            .map(|_| r.id)
                    })
                    .collect(),
            ),
        };

        let mut pieces = Vec::new();
        let mut total = 0;
        for rec in self
            .store
            .list_by_date(KIND_ENTRY, q.from.as_deref(), q.to.as_deref())?
        {
            let e: TimeEntry = payload(&rec)?;
            let keep = text
                .as_ref()
                .is_none_or(|t| e.description.to_lowercase().contains(t))
                && q.project_id.as_ref().is_none_or(|p| *p == e.project_id)
                && q.category_id
                    .as_ref()
                    .is_none_or(|c| Some(c) == e.category_id.as_ref())
                && q.billable.is_none_or(|b| b == e.billable)
                && tag
                    .as_ref()
                    .is_none_or(|t| e.tags.iter().any(|x| x.to_lowercase() == *t))
                && client_projects
                    .as_ref()
                    .is_none_or(|ps| ps.contains(&e.project_id));
            if keep {
                total += e.minutes;
                pieces.push(rec);
            }
        }
        pieces.reverse();
        Ok(SearchResult {
            pieces,
            total_minutes: total,
        })
    }
}
