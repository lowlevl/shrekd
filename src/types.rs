use rocket::data::ByteUnit;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

use thiserror::Error;

pub const STORAGE_PREFIX: &str = "shrt";

/** Represent's an application's error */
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Configuration Error: {0}")]
    Config(#[from] figment::Error),

    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization or Deserialization Error: {0}")]
    SerDe(#[from] serde_json::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request<'_>,
    ) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::Response};
        use std::io::Cursor;

        let status = match self {
            Error::IO(_) => Status::InternalServerError,
            Error::Config(_) => Status::InternalServerError,
            Error::Redis(_) => Status::InternalServerError,
            Error::SerDe(_) => Status::InternalServerError,
        };
        let output = self.to_string();

        Ok(Response::build()
            .status(status)
            .sized_body(output.len(), Cursor::new(output))
            .finalize())
    }
}

/** Convenience alias of [`std::result::Result`] with the [`Error`] prefilled */
pub type Result<T, E = Error> = std::result::Result<T, E>;

/** Represents a record with it's params and data */
#[derive(Serialize, Deserialize)]
pub struct Record {
    /** Actual [`Record`] data, or access to it */
    data: RecordData,
    /** [`Record`]'s random *secret* slug */
    slug: String,
    /** Remaining number of accesses, if applicable */
    accesses: Option<usize>,
    /** Date of expiry, if applicable */
    expiry: Option<usize>,
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            RecordData::File { path, size } => {
                write!(f, "Record::File<{:?}, {}>", path, ByteUnit::from(*size))
            }
            RecordData::Redirect { target } => write!(f, "Record::Redirect<{}>", target),
            RecordData::Paste { body } => write!(f, "Record::Paste<{} chars>", body.len()),
        }?;

        write!(
            f,
            " {{ accesses: {:?}, expiry: {:?} }}",
            self.accesses, self.expiry
        )
    }
}

impl Record {
    pub fn file(
        path: PathBuf,
        size: usize,
        slug: String,
        accesses: Option<usize>,
        expiry: Option<usize>,
    ) -> Self {
        Record {
            data: RecordData::File { path, size },
            slug,
            accesses,
            expiry,
        }
    }

    fn key(slug: &str) -> String {
        format!("{}:{}", STORAGE_PREFIX, slug)
    }

    pub async fn push(&self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
        use redis::AsyncCommands;

        Ok(conn
            .set(Self::key(&self.slug), serde_json::to_string(self)?)
            .await?)
    }
}

/** Represents a record's data, or a link to it */
#[derive(Debug, Serialize, Deserialize)]
pub enum RecordData {
    /** Represents a stored file, see [`Record`] */
    File {
        path: PathBuf,
        size: usize,
    },
    /** Represents a URL redirect, see [`Record`] */
    Redirect {
        target: Url,
    },
    /* Represents a paste in utf-8, see [`Record`] */
    Paste {
        body: String,
    },
}
