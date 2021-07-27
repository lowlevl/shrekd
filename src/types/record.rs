use chrono::{DateTime, Utc};
use rocket::{
    data::ByteUnit,
    request::{self, FromRequest, Request},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::{Result, STORAGE_PREFIX};

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

    /** Instanciate a new `Redirect`-variant record */
    pub fn redirect(
        url: rocket::http::uri::Absolute<'static>,
        slug: String,
        accesses: Option<u16>,
        expiry: Option<DateTime<Utc>>,
    ) -> Self {
        Record {
            data: RecordData::Redirect { to: url },
            slug,
            accesses,
            expiry,
        }
    }

    fn key(slug: &str) -> String {
        [STORAGE_PREFIX, slug].concat()
    }

    /** Access the [`Record`]'s [`RecordData`] */
    pub fn data(&self) -> &RecordData {
        &self.data
    }

    /** Access the [`Record`]'s `slug` */
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /** Persist the [`Record`] to the Redis server */
    pub async fn persist(&self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
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
    pub async fn fetch(
        slug: &str,
        conn: &mut redis::aio::Connection,
    ) -> crate::Result<Option<Self>> {
        use redis::AsyncCommands;

        Ok(conn
            .get::<_, Option<String>>(Self::key(slug))
            .await?
            .map(|record| serde_json::from_str(&record))
            .transpose()?)
    }

    /** Consume this instance of the [`Record`], and update it's intrinsics to reflect the fact it has been accessed */
    pub async fn consume(self, conn: &mut redis::aio::Connection) -> crate::Result<()> {
        let record = Record {
            /* Register a new access if needed */
            accesses: self.accesses.map(|count| count - 1),
            ..self
        };

        match record.accesses {
            Some(0) => {
                log::trace!("Record has no accesses left, removing");
                record.delete(&mut *conn).await?
            }
            Some(count) => {
                log::trace!("Record has `{}` accesses left, pushing change", count);
                record.persist(&mut *conn).await?
            }
            None => (),
        };

        Ok(())
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
        to: rocket::http::uri::Absolute<'static>,
    },
    /* Represents a paste in utf-8, see [`Record`] */
    Paste {
        body: String,
    },
}

/** Structure representing parameters regarding the configuration of [`Record`]s */
#[derive(Debug)]
pub struct RecordSettings {
    /** Maximum number of accesses before the removal of the record */
    max_access: Option<u16>,
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

        let max_access = match req
            .headers()
            .get_one("Max-Access")
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
            max_access,
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
        self.max_access
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
