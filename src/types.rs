use chrono::{DateTime, Utc};
use rocket::{
    data::ByteUnit,
    request::{self, FromRequest, Request},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use thiserror::Error;

/** The storage prefix for keys on Redis */
pub const STORAGE_PREFIX: &str = "shrt:";

/** Represent's an application's error */
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum Error {
    /* 4xx errors */
    #[error("Couldn't find the record identified with the slug `{0}`")]
    NotFound(String),

    #[error("File upload failed ({0})")]
    FileUpload(String),

    #[error("Paste creation failed ({0})")]
    PasteCreation(String),

    /* 5xx errors */
    #[error("There's an error with the configuration ({0})")]
    Config(#[from] figment::Error),

    #[error("I/O error: {0}")]
    IO(#[from] tokio::io::Error),

    #[error("Could not query the Redis server ({0})")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization or deserialization error ({0})")]
    SerDe(#[from] serde_json::Error),
}

impl<'r, 'o: 'r> rocket::response::Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r rocket::request::Request<'_>) -> rocket::response::Result<'o> {
        use rocket::{http::Status, response::status, serde::json};

        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let error = json::Json(ErrorResponse {
            message: self.to_string(),
        });

        Ok(match self {
            /* 4xx errors */
            Error::NotFound(_) => status::NotFound(error).respond_to(req)?,
            Error::PasteCreation(_) | Error::FileUpload(_) => {
                status::Custom(Status::UnprocessableEntity, error).respond_to(req)?
            }

            /* 5xx errors */
            Error::Redis(_) => status::Custom(Status::ServiceUnavailable, error).respond_to(req)?,
            Error::Config(_) | Error::IO(_) | Error::SerDe(_) => {
                status::Custom(Status::InternalServerError, error).respond_to(req)?
            }
        })
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
    accesses: Option<u16>,
    /** Date of expiry, if applicable */
    expiry: Option<DateTime<Utc>>,
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.data {
            RecordData::File { path, size } => {
                write!(f, "Record::File<{:?}, {}>", path, ByteUnit::from(*size))
            }
            RecordData::Redirect { to } => write!(f, "Record::Redirect<{}>", to),
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
        accesses: Option<u16>,
        expiry: Option<DateTime<Utc>>,
    ) -> Self {
        Record {
            data: RecordData::File { path, size },
            slug,
            accesses,
            expiry,
        }
    }

    /** Instanciate a new `Paste`-variant record */
    pub fn paste(
        data: String,
        slug: String,
        accesses: Option<u16>,
        expiry: Option<DateTime<Utc>>,
    ) -> Self {
        Record {
            data: RecordData::Paste { body: data },
            slug,
            accesses,
            expiry,
        }
    }

    fn key(slug: &str) -> String {
        [STORAGE_PREFIX, slug].concat()
    }

    /** Access the underlying [`RecordData`] */
    pub fn data(&self) -> &RecordData {
        &self.data
    }

    /** Push the [`Record`] to the Redis server */
    pub async fn push(&self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
        use redis::AsyncCommands;

        /* Push the Record into Redis */
        conn.set(Self::key(&self.slug), serde_json::to_string(self)?)
            .await?;

        if let Some(expiry) = self.expiry {
            /* Set it's expiry if required */
            conn.expire_at(Self::key(&self.slug), expiry.timestamp() as usize)
                .await?
        }

        Ok(())
    }

    /** Delete the [`Record`] from the Redis server */
    pub async fn delete(&self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
        use redis::AsyncCommands;

        Ok(conn.del(Self::key(&self.slug)).await?)
    }

    /** Pull a [`Record`] from the Redis server from it's `slug` */
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

        log::trace!("Retrieved the following data `{:?}` from Redis", record);

        Ok(match record {
            Some(record) => {
                let record = Record {
                    /* Register a new access if needed */
                    accesses: record.accesses.map(|count| count - 1),
                    ..record
                };

                match record.accesses {
                    Some(0) => {
                        log::trace!("Record has no accesses left, removing");
                        record.delete(&mut *conn).await?
                    }
                    Some(count) => {
                        log::trace!("Record has `{}` accesses left, pushing change", count);
                        record.push(&mut *conn).await?
                    }
                    None => (),
                };

                Some(record)
            }
            _ => None,
        })
    }

    /** Checks for the existence of a [`Record`] from it's `slug` in the server */
    pub async fn exists(slug: &str, conn: &mut redis::aio::Connection) -> Result<bool> {
        use redis::AsyncCommands;

        Ok(conn.exists(Self::key(slug)).await?)
    }
}

/** Represents a record's data, or a link to it */
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RecordData {
    /** Represents a stored file, see [`Record`] */
    File {
        path: PathBuf,
        size: usize,
    },
    /** Represents a URL redirect, see [`Record`] */
    Redirect {
        to: rocket::http::uri::Reference<'static>,
    },
    /* Represents a paste in utf-8, see [`Record`] */
    Paste {
        body: String,
    },
}

/** Structure representing parameters regarding the configuration of [`Record`]s */
#[derive(Debug)]
pub struct RecordSettings {
    /** Maximum number of downloads before the removal of the record */
    max_downloads: Option<u16>,
    /** Utc timestamp of removal of the record */
    expiry_timestamp: Option<i64>,
    /** Expiry time from now, in seconds of the record */
    expire_in: Option<i64>,
    /** Desired `slug` length */
    slug_length: Option<u8>,
    /** Desired custom `slug` */
    custom_slug: Option<String>,
    /** Checksum of the record to be verified upon upload */
    data_checksum: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RecordSettings {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        use rocket::http::Status;

        let max_downloads = match req
            .headers()
            .get_one("Max-Downloads")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        let expiry_timestamp = match req
            .headers()
            .get_one("Expiry-Timestamp")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        let expire_in = match req
            .headers()
            .get_one("Expire-In")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        let slug_length = match req
            .headers()
            .get_one("Slug-Length")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        let custom_slug = match req
            .headers()
            .get_one("Custom-Slug")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        let data_checksum = match req
            .headers()
            .get_one("Data-Checksum")
            .map(str::parse)
            .transpose()
        {
            Ok(data) => data,
            Err(_) => return request::Outcome::Failure((Status::BadRequest, ())),
        };

        /* If the two collide, return a Failure, both cannot be defined at the same time */
        if expiry_timestamp.is_some() && expire_in.is_some() {
            return request::Outcome::Failure((Status::BadRequest, ()));
        }

        request::Outcome::Success(RecordSettings {
            max_downloads,
            expiry_timestamp,
            expire_in,
            slug_length,
            custom_slug,
            data_checksum,
        })
    }
}

impl RecordSettings {
    /** Extract the number of accesses from the [`RecordSettings`] */
    pub fn accesses(&self) -> Option<u16> {
        self.max_downloads
    }

    /** Compute the expiry from the [`RecordSettings`] */
    pub fn expiry(&self) -> Option<DateTime<Utc>> {
        use chrono::{Duration, NaiveDateTime};

        match (self.expiry_timestamp, self.expire_in) {
            (Some(expiry_timestamp), _) => Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(expiry_timestamp, 0),
                Utc,
            )),
            (_, Some(expire_in)) => Some(Utc::now() + Duration::seconds(expire_in)),
            _ => None,
        }
    }

    /** Compute the slug from the [`RecordSettings`] and [`Config`] and ensure it's not colliding */
    pub async fn slug(
        &self,
        config: &crate::Config,
        conn: &mut redis::aio::Connection,
    ) -> Result<String> {
        use rand::{distributions::Alphanumeric, Rng};

        Ok(match self.custom_slug {
            /* If a custom slug exists, is not empty and does not exist, use it */
            Some(ref slug) if !slug.is_empty() && !Record::exists(slug, &mut *conn).await? => {
                slug.clone()
            }
            /* Else, generate a random slug of `max(<slug configured length>, <desired length>)` */
            _ => {
                let length =
                    std::cmp::max(config.slug_length, self.slug_length.unwrap_or_default());

                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(length as usize)
                    .map(char::from)
                    .collect()
            }
        })
    }
}
