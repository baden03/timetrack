//! The application service: the one place where commands are validated and
//! applied. The desktop shell (and later the hub and a public API) call this
//! and nothing underneath it. It never touches the user interface.

mod dispatch;
mod entries;
mod error;
mod search;
mod service;
mod summary;
mod timer;

pub use entries::{EntryPatch, NewEntry, SavedEntry, Suggestion, Timing};
pub use error::{AppError, Result};
pub use search::{SearchQuery, SearchResult};
pub use service::{system_clock, App, AppState, Clock, Identity, ProjectInput};
pub use summary::MonthlySummary;
pub use timer::{StartTimer, StopTimer, TimerState};
