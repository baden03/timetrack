use timetrack_core::id::IdError;
use timetrack_core::model::ModelError;
use timetrack_core::record::RecordError;
use timetrack_core::time::TimeError;
use timetrack_storage::StoreError;

/// Everything a command can refuse with. The UI shows `error.<code>` from the
/// message catalog.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Time(#[from] TimeError),
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error(transparent)]
    Record(#[from] RecordError),
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("invalid date or time")]
    InvalidDateTime,
    #[error("unknown time zone")]
    UnknownTimeZone,
    #[error("record not found")]
    NotFound,
    #[error("record is of a different kind")]
    WrongKind,
    #[error("project is archived")]
    ProjectArchived,
    #[error("category is archived")]
    CategoryArchived,
    #[error("client is archived")]
    ClientArchived,
    #[error("category is not enabled for the project")]
    CategoryNotEnabled,
    #[error("still referenced")]
    StillReferenced,
    #[error("unknown command")]
    UnknownCommand,
    #[error("invalid arguments")]
    InvalidArguments,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Time(e) => e.code(),
            AppError::Model(e) => e.code(),
            AppError::Record(e) => e.code(),
            AppError::Id(e) => e.code(),
            AppError::Store(e) => e.code(),
            AppError::InvalidDateTime => "invalid_date_time",
            AppError::UnknownTimeZone => "unknown_time_zone",
            AppError::NotFound => "not_found",
            AppError::WrongKind => "wrong_kind",
            AppError::ProjectArchived => "project_archived",
            AppError::CategoryArchived => "category_archived",
            AppError::ClientArchived => "client_archived",
            AppError::CategoryNotEnabled => "category_not_enabled",
            AppError::StillReferenced => "still_referenced",
            AppError::UnknownCommand => "unknown_command",
            AppError::InvalidArguments => "invalid_arguments",
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
