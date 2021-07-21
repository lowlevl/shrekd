use chrono::{DateTime, Utc};
use rocket::data::ByteUnit;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

use thiserror::Error;

pub const STORAGE_PREFIX: &str = "shrt";

/** Represent's an application's error */
#[derive(Error, Debug)]
pub enum Error {
    /* 4xx errors */
    #[error("Could not found the record identified with the slug `{0}`")]
    NotFound(String),

    /* 5xx errors */
    #[error("Configuration Error: {0}")]
    Config(#[from] figment::Error),

    #[error("IO Error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Ser/De Error: {0}")]
    SerDe(#[from] serde_json::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(
        self,
        _request: &'r rocket::request::Request<'_>,
    ) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::Response};
        use std::io::Cursor;

        // #[derive(Serialize, Deserialize)]
        // struct ErrorResponse {
        //     message: String,
        // }

        let status = match self {
            /* 4xx errors */
            Error::NotFound(_) => Status::NotFound,

            /* 5xx errors */
            Error::Config(_) => Status::InternalServerError,
            Error::Redis(_) => Status::ServiceUnavailable,
            Error::IO(_) => Status::InternalServerError,
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
#[derive(Serialize, Deserialize, Clone)]
pub struct Record {
    /** Actual [`Record`] data, or access to it */
    data: RecordData,
    /** [`Record`]'s random *secret* slug */
    slug: String,
    /** Remaining number of accesses, if applicable */
    accesses: Option<usize>,
    /** Date of expiry, if applicable */
    expiry: Option<DateTime<Utc>>,
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
    /** Instanciate a new `File`-variant record */
    pub fn file(
        path: PathBuf,
        size: usize,
        slug: String,
        accesses: Option<usize>,
        expiry: Option<DateTime<Utc>>,
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

    /** Access the underlying [`RecordData`] */
    pub fn data(&self) -> &RecordData {
        &self.data
    }

    /** Push the [`Record`] to the Redis server */
    pub async fn push(&self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
        use redis::AsyncCommands;

        Ok(conn
            .set(Self::key(&self.slug), serde_json::to_string(self)?)
            .await?)
    }

    /** Pull the [`Record`] from the Redis server */
    pub async fn pull(
        slug: &str,
        conn: &mut redis::aio::Connection,
    ) -> crate::Result<Option<Self>> {
        use redis::AsyncCommands;

        let record: Option<Record> = conn
            .get::<_, Option<String>>(Self::key(slug))
            .await?
            .map(|record| serde_json::from_str(&record))
            .transpose()?;

        log::trace!("Retrieved the following data {:?} from Redis", record);

        Ok(match record {
            Some(record) => match (record.accesses, record.expiry) {
                /* Guard against the accesses count */
                (Some(accesses), _) if accesses == 0 => {
                    log::trace!("Record found, but there's no accesses left");

                    None
                }
                /* Guard against the expiry */
                (_, Some(expiry)) if expiry >= Utc::now() => {
                    log::trace!("Record found, but it is expired");

                    None
                }
                /* If an access counter is defined, consume one */
                (Some(_accesses), _) => {
                    let record = Record {
                        accesses: record.accesses.map(|count| count - 1),
                        ..record
                    };
                    record.push(&mut *conn).await?;

                    Some(record)
                }
                _ => Some(record),
            },
            _ => None,
        })
    }
}

/** Represents a record's data, or a link to it */
#[derive(Serialize, Deserialize, Debug, Clone)]
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
