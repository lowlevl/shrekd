mod error;
mod host;
mod record;
mod retention;

/** The storage prefix for keys on Redis */
pub const STORAGE_PREFIX: &str = "shrekd:";

pub use {
    error::{Error, Result},
    host::HostBase,
    record::{Record, RecordData, RecordSettings},
    retention::RetentionCurve,
};
