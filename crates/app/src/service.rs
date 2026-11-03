use std::sync::Arc;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use timetrack_core::id::{Id, IdGenerator, Origin};
use timetrack_core::model::{
    normalize_color, require_name, Category, Client, Project, Settings, Status, Theme,
    KIND_CATEGORY, KIND_CLIENT, KIND_ENTRY, KIND_PROJECT, KIND_SETTINGS,
};
use timetrack_core::record::{Actor, HistoryEntry, Record};
use timetrack_storage::Store;

use crate::error::{AppError, Result};

pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

pub fn system_clock() -> Clock {
    Arc::new(Utc::now)
}

#[derive(Debug, Clone, Serialize)]
pub struct Identity {
    pub origin: String,
    pub author: String,
    pub device: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppState {
    pub first_run: bool,
    /// When the last backup was written, if ever.
    pub last_backup_at: Option<String>,
    pub identity: Identity,
    pub settings: Record,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectInput {
    pub client_id: Id,
    pub name: String,
    pub description: String,
    pub billable_default: bool,
    pub category_ids: Vec<Id>,
    /// Optional `#rrggbb`.
    #[serde(default)]
    pub color: Option<String>,
}

pub struct App<S: Store> {
    pub(crate) store: S,
    pub(crate) ids: IdGenerator,
    pub(crate) actor: Actor,
    pub(crate) clock: Clock,
    settings_id: Id,
    system_zone: String,
}

impl<S: Store> App<S> {
    /// Opens (or on first launch, initializes) an installation. `system_zone`
    /// is the device's IANA zone, used as the default for new entries.
    pub fn open(mut store: S, system_zone: &str, clock: Clock) -> Result<Self> {
        let (ids, actor, settings_id) = bootstrap(&mut store, system_zone, &clock)?;
        Ok(App {
            store,
            ids,
            actor,
            clock,
            settings_id,
            system_zone: system_zone.to_owned(),
        })
    }

    pub fn now(&self) -> DateTime<Utc> {
        (self.clock)()
    }

    pub fn identity(&self) -> Identity {
        Identity {
            origin: self.ids.origin().to_string(),
            author: self.actor.author.clone(),
            device: self.actor.device.clone(),
        }
    }

    pub fn state(&self) -> Result<AppState> {
        Ok(AppState {
            first_run: self.store.meta_get("first_run_done")?.is_none(),
            last_backup_at: self.store.meta_get("last_backup_at")?,
            identity: self.identity(),
            settings: self.settings()?,
        })
    }

    /// Writes a backup to one file and remembers when (concept §16.2).
    pub fn backup(&mut self, path: &std::path::Path) -> Result<timetrack_storage::BackupInfo> {
        // The time is stored first so the backup file carries it: after a
        // restore, "last backup" is the time of the file that was restored.
        let previous = self.store.meta_get("last_backup_at")?;
        let stamp = self
            .now()
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.store.meta_set("last_backup_at", &stamp)?;
        match self.store.backup_to(path) {
            Ok(info) => Ok(info),
            Err(e) => {
                if let Some(old) = previous {
                    self.store.meta_set("last_backup_at", &old)?;
                }
                Err(e.into())
            }
        }
    }

    /// What a backup file holds, to confirm before replacing the current data.
    pub fn inspect_backup(&self, path: &std::path::Path) -> Result<timetrack_storage::BackupInfo> {
        Ok(S::inspect_backup(path)?)
    }

    /// Replaces all data with a backup. The current data is first copied to
    /// `safety_copy`, so a mistaken restore can be undone; a file that is not
    /// a valid backup is refused before anything changes.
    pub fn restore(
        &mut self,
        path: &std::path::Path,
        safety_copy: &std::path::Path,
    ) -> Result<timetrack_storage::BackupInfo> {
        S::inspect_backup(path)?;
        self.store.backup_to(safety_copy)?;
        let info = self.store.restore_from(path)?;
        let (ids, actor, settings_id) = bootstrap(&mut self.store, &self.system_zone, &self.clock)?;
        self.ids = ids;
        self.actor = actor;
        self.settings_id = settings_id;
        Ok(info)
    }

    /// Records the stand-alone choice (architecture 5.9a).
    pub fn finish_first_run(&mut self) -> Result<()> {
        self.store.meta_set("first_run_done", "standalone")?;
        Ok(())
    }

    // ---- settings -------------------------------------------------------

    pub fn settings(&self) -> Result<Record> {
        self.load(&self.settings_id, KIND_SETTINGS)
    }

    pub fn settings_value(&self) -> Result<Settings> {
        payload(&self.settings()?)
    }

    pub fn update_settings(&mut self, new: Settings) -> Result<Record> {
        new.timezone
            .parse::<Tz>()
            .map_err(|_| AppError::UnknownTimeZone)?;
        if !timetrack_core::i18n::is_supported(&new.language) {
            return Err(AppError::UnsupportedLanguage);
        }
        let rec = self.settings()?;
        self.revise(&rec, &new)
    }

    // ---- clients, categories, projects ---------------------------------

    pub fn create_client(&mut self, name: &str, color: Option<&str>) -> Result<Record> {
        let payload = Client {
            name: require_name(name)?,
            color: normalize_color(color)?,
            status: Status::Active,
        };
        self.create(KIND_CLIENT, &payload)
    }

    pub fn rename_client(&mut self, id: &Id, name: &str) -> Result<Record> {
        let rec = self.load(id, KIND_CLIENT)?;
        let mut c: Client = payload(&rec)?;
        c.name = require_name(name)?;
        self.revise(&rec, &c)
    }

    /// Sets or removes (`None`) the client's color.
    pub fn set_client_color(&mut self, id: &Id, color: Option<&str>) -> Result<Record> {
        let rec = self.load(id, KIND_CLIENT)?;
        let mut c: Client = payload(&rec)?;
        c.color = normalize_color(color)?;
        self.revise(&rec, &c)
    }

    pub fn set_client_status(&mut self, id: &Id, status: Status) -> Result<Record> {
        let rec = self.load(id, KIND_CLIENT)?;
        let mut c: Client = payload(&rec)?;
        c.status = status;
        self.revise(&rec, &c)
    }

    /// Hard delete, allowed only while no project belongs to the client.
    pub fn delete_client(&mut self, id: &Id) -> Result<()> {
        let rec = self.load(id, KIND_CLIENT)?;
        for p in self.store.list(KIND_PROJECT, false)? {
            if payload::<Project>(&p)?.client_id == *id {
                return Err(AppError::StillReferenced);
            }
        }
        self.tombstone(&rec)
    }

    pub fn create_category(&mut self, name: &str) -> Result<Record> {
        let payload = Category {
            name: require_name(name)?,
            status: Status::Active,
        };
        self.create(KIND_CATEGORY, &payload)
    }

    pub fn rename_category(&mut self, id: &Id, name: &str) -> Result<Record> {
        let rec = self.load(id, KIND_CATEGORY)?;
        let mut c: Category = payload(&rec)?;
        c.name = require_name(name)?;
        self.revise(&rec, &c)
    }

    pub fn set_category_status(&mut self, id: &Id, status: Status) -> Result<Record> {
        let rec = self.load(id, KIND_CATEGORY)?;
        let mut c: Category = payload(&rec)?;
        c.status = status;
        self.revise(&rec, &c)
    }

    /// Hard delete, never while a project enables it or an entry uses it.
    pub fn delete_category(&mut self, id: &Id) -> Result<()> {
        let rec = self.load(id, KIND_CATEGORY)?;
        for p in self.store.list(KIND_PROJECT, false)? {
            if payload::<Project>(&p)?.category_ids.contains(id) {
                return Err(AppError::StillReferenced);
            }
        }
        for e in self.store.list_by_date(KIND_ENTRY, None, None)? {
            if payload::<timetrack_core::model::TimeEntry>(&e)?
                .category_id
                .as_ref()
                == Some(id)
            {
                return Err(AppError::StillReferenced);
            }
        }
        self.tombstone(&rec)
    }

    pub fn create_project(&mut self, input: ProjectInput) -> Result<Record> {
        let project = self.project_from_input(input, Status::Active)?;
        self.create(KIND_PROJECT, &project)
    }

    pub fn update_project(&mut self, id: &Id, input: ProjectInput) -> Result<Record> {
        let rec = self.load(id, KIND_PROJECT)?;
        let old: Project = payload(&rec)?;
        let project = self.project_from_input(input, old.status)?;
        self.revise(&rec, &project)
    }

    pub fn set_project_status(&mut self, id: &Id, status: Status) -> Result<Record> {
        let rec = self.load(id, KIND_PROJECT)?;
        let mut p: Project = payload(&rec)?;
        p.status = status;
        self.revise(&rec, &p)
    }

    /// Hard delete, allowed only while the project has no entries.
    pub fn delete_project(&mut self, id: &Id) -> Result<()> {
        let rec = self.load(id, KIND_PROJECT)?;
        for e in self.store.list_by_date(KIND_ENTRY, None, None)? {
            if payload::<timetrack_core::model::TimeEntry>(&e)?.project_id == *id {
                return Err(AppError::StillReferenced);
            }
        }
        self.tombstone(&rec)
    }

    /// Records of a reference kind (client, project, category), oldest first.
    pub fn list(&self, kind: &str, include_archived: bool) -> Result<Vec<Record>> {
        if ![KIND_CLIENT, KIND_PROJECT, KIND_CATEGORY].contains(&kind) {
            return Err(AppError::WrongKind);
        }
        let mut records = self.store.list(kind, false)?;
        if !include_archived {
            records.retain(|r| r.payload["status"] != "archived");
        }
        Ok(records)
    }

    pub fn history(&self, id: &Id) -> Result<Vec<HistoryEntry>> {
        Ok(self.store.history(id)?)
    }

    fn project_from_input(&self, input: ProjectInput, status: Status) -> Result<Project> {
        let client: Client = payload(&self.load(&input.client_id, KIND_CLIENT)?)?;
        if client.status == Status::Archived {
            return Err(AppError::ClientArchived);
        }
        for c in &input.category_ids {
            self.load(c, KIND_CATEGORY)?;
        }
        Ok(Project {
            client_id: input.client_id,
            name: require_name(&input.name)?,
            color: normalize_color(input.color.as_deref())?,
            description: input.description.trim().to_owned(),
            billable_default: input.billable_default,
            category_ids: input.category_ids,
            status,
        })
    }

    // ---- record plumbing ------------------------------------------------

    pub(crate) fn load(&self, id: &Id, kind: &str) -> Result<Record> {
        match self.store.get(id)? {
            Some(r) if r.deleted => Err(AppError::NotFound),
            Some(r) if r.kind != kind => Err(AppError::WrongKind),
            Some(r) => Ok(r),
            None => Err(AppError::NotFound),
        }
    }

    pub(crate) fn create<T: Serialize>(&mut self, kind: &str, value: &T) -> Result<Record> {
        let (rec, hist) = Record::create(
            &self.ids,
            kind,
            to_value(value),
            self.actor.clone(),
            self.now(),
        );
        self.store.put(&rec, &hist)?;
        Ok(rec)
    }

    pub(crate) fn revise<T: Serialize>(&mut self, rec: &Record, value: &T) -> Result<Record> {
        let (next, hist) = rec.revise(to_value(value), self.actor.clone(), self.now())?;
        self.store.put(&next, &hist)?;
        Ok(next)
    }

    fn tombstone(&mut self, rec: &Record) -> Result<()> {
        let (next, hist) = rec.delete(self.actor.clone(), self.now())?;
        self.store.put(&next, &hist)?;
        Ok(())
    }
}

/// Reads or creates the installation's identity and settings.
fn bootstrap<S: Store>(
    store: &mut S,
    system_zone: &str,
    clock: &Clock,
) -> Result<(IdGenerator, Actor, Id)> {
    let origin = match store.meta_get("origin")? {
        Some(o) => Origin::new(&o)?,
        None => {
            let o = Origin::random();
            store.meta_set("origin", o.as_str())?;
            o
        }
    };
    let ids = IdGenerator::new(origin);
    let author = meta_or_new(store, &ids, "author")?;
    let device = meta_or_new(store, &ids, "device")?;
    let actor = Actor { author, device };

    let settings_id = match store.meta_get("settings_id")? {
        Some(id) => Id::parse(&id)?,
        None => {
            let zone = if system_zone.parse::<Tz>().is_ok() {
                system_zone
            } else {
                "UTC"
            };
            let payload = Settings {
                language: "en".into(),
                timezone: zone.into(),
                date_format: "DD.MM.YYYY".into(),
                clock_24h: true,
                theme: Theme::System,
                entry_window_days: 60,
                backup_reminder_days: 7,
            };
            let (rec, hist) = Record::create(
                &ids,
                KIND_SETTINGS,
                serde_json::to_value(payload).expect("settings serialize"),
                actor.clone(),
                clock(),
            );
            store.put(&rec, &hist)?;
            store.meta_set("settings_id", rec.id.as_str())?;
            rec.id
        }
    };
    Ok((ids, actor, settings_id))
}

pub(crate) fn payload<T: DeserializeOwned>(rec: &Record) -> Result<T> {
    serde_json::from_value(rec.payload.clone())
        .map_err(|e| AppError::Store(timetrack_storage::StoreError::Storage(e.to_string())))
}

pub(crate) fn to_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).expect("payload serializes")
}

fn meta_or_new<S: Store>(store: &mut S, ids: &IdGenerator, key: &str) -> Result<String> {
    if let Some(v) = store.meta_get(key)? {
        return Ok(v);
    }
    let v = ids.next_id().to_string();
    store.meta_set(key, &v)?;
    Ok(v)
}
