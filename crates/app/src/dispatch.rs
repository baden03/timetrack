//! A single entry point for the interface: a command name plus JSON
//! arguments in, JSON out. The desktop shell forwards its one `call`
//! command here, and a future hub or API can do the same.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use timetrack_core::id::Id;
use timetrack_core::model::{Settings, Status};
use timetrack_storage::Store;

use crate::entries::{EntryPatch, NewEntry};
use crate::error::{AppError, Result};
use crate::service::{App, ProjectInput};

fn arg<T: DeserializeOwned>(args: &Value, name: &str) -> Result<T> {
    let value = args.get(name).ok_or(AppError::InvalidArguments)?;
    serde_json::from_value(value.clone()).map_err(|_| AppError::InvalidArguments)
}

fn opt<T: DeserializeOwned>(args: &Value, name: &str) -> Result<Option<T>> {
    match args.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => serde_json::from_value(v.clone())
            .map(Some)
            .map_err(|_| AppError::InvalidArguments),
    }
}

fn out<T: Serialize>(value: T) -> Result<Value> {
    Ok(serde_json::to_value(value).expect("results serialize"))
}

impl<S: Store> App<S> {
    pub fn dispatch(&mut self, command: &str, args: &Value) -> Result<Value> {
        match command {
            "state" => out(self.state()?),
            "finish_first_run" => out(self.finish_first_run()?),
            "update_settings" => out(self.update_settings(arg::<Settings>(args, "settings")?)?),
            "list" => out(self.list(
                &arg::<String>(args, "kind")?,
                opt(args, "include_archived")?.unwrap_or(false),
            )?),

            "create_client" => out(self.create_client(&arg::<String>(args, "name")?)?),
            "rename_client" => {
                out(self.rename_client(&arg::<Id>(args, "id")?, &arg::<String>(args, "name")?)?)
            }
            "set_client_status" => {
                out(self
                    .set_client_status(&arg::<Id>(args, "id")?, arg::<Status>(args, "status")?)?)
            }
            "delete_client" => out(self.delete_client(&arg::<Id>(args, "id")?)?),

            "create_category" => out(self.create_category(&arg::<String>(args, "name")?)?),
            "rename_category" => {
                out(self.rename_category(&arg::<Id>(args, "id")?, &arg::<String>(args, "name")?)?)
            }
            "set_category_status" => {
                out(self
                    .set_category_status(&arg::<Id>(args, "id")?, arg::<Status>(args, "status")?)?)
            }
            "delete_category" => out(self.delete_category(&arg::<Id>(args, "id")?)?),

            "create_project" => out(self.create_project(arg::<ProjectInput>(args, "input")?)?),
            "update_project" => {
                out(self
                    .update_project(&arg::<Id>(args, "id")?, arg::<ProjectInput>(args, "input")?)?)
            }
            "set_project_status" => {
                out(self
                    .set_project_status(&arg::<Id>(args, "id")?, arg::<Status>(args, "status")?)?)
            }
            "delete_project" => out(self.delete_project(&arg::<Id>(args, "id")?)?),

            "create_entry" => out(self.create_entry(arg::<NewEntry>(args, "entry")?)?),
            "update_entry" => out(self.update_entry(
                &arg::<String>(args, "group")?,
                arg::<EntryPatch>(args, "patch")?,
            )?),
            "delete_entry" => out(self.delete_entry(&arg::<String>(args, "group")?)?),
            "list_entries" => {
                let from: Option<String> = opt(args, "from")?;
                let to: Option<String> = opt(args, "to")?;
                out(self.list_entries(from.as_deref(), to.as_deref())?)
            }
            "suggest" => out(self.suggest(
                &arg::<Id>(args, "project_id")?,
                &arg::<String>(args, "date")?,
            )?),
            "history" => out(self.history(&arg::<Id>(args, "id")?)?),
            _ => Err(AppError::UnknownCommand),
        }
    }
}
