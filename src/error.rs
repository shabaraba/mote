use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoteError {
    #[error("Mote is not initialized. Run 'mote init' first.")]
    NotInitialized,

    #[error("Mote is already initialized in this directory.")]
    AlreadyInitialized,

    #[error("No VCS directory found (.git or .jj). Required for location_strategy = 'vcs'.")]
    NoVcsDirectory,

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("No snapshots available")]
    NoSnapshotsAvailable,

    #[error("Ambiguous snapshot ID: {0}. Multiple matches found.")]
    AmbiguousSnapshotId(String),

    #[error("File not found in snapshot: {0}")]
    FileNotFoundInSnapshot(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Object hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Failed to read config: {0}")]
    ConfigRead(String),

    #[error("Failed to parse config: {0}")]
    ConfigParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Context not found: {0}")]
    ContextNotFound(String),

    #[error("Context already exists: {0}")]
    ContextAlreadyExists(String),

    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
}

pub type Result<T> = std::result::Result<T, MoteError>;
