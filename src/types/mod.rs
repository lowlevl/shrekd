mod error;
mod record;

/** The storage prefix for keys on Redis */
pub const STORAGE_PREFIX: &str = "shrtd:";

pub use error::{Error, Result};
pub use record::{Record, RecordData, RecordSettings};
