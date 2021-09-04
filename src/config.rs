use figment::{Figment, Metadata, Profile, Provider};
use rocket::data::ToByteUnit;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{types::RetentionCurve, Error, Result};

const TEMPDIR_NAME: &str = ".temporary";

/** Global configuration structure */
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /** Address on which the server will be exposed */
    pub address: String,
    /** Port on which the server will be exposed */
    pub port: u16,

    /** Redis server URL */
    pub redis_url: String,
    /** App's permanent data storage directory */
    pub data_dir: PathBuf,
    /** Random URI's slug length, in characters */
    pub slug_length: u8,

    /** Max file size, in bytes */
    pub max_file_size: u64,
    /** Max paste size, in bytes */
    pub max_paste_size: u64,
    /** Max url size, in bytes */
    pub max_url_size: u64,

    /** Retention min age, in seconds */
    pub retention_min_age: u64,
    /** Retention max age, in seconds */
    pub retention_max_age: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: String::from("0.0.0.0"),
            port: 8000,
            redis_url: String::from("redis://127.0.0.1:6379"),
            data_dir: PathBuf::from("/tmp/.shrekd"),
            slug_length: 13,
            max_file_size: 128.megabytes().into(),
            max_paste_size: 1.megabytes().into(),
            max_url_size: 32.kilobytes().into(),
            retention_min_age: 60 * 60 * 24 * 7,     /* 1 week */
            retention_max_age: 60 * 60 * 24 * 7 * 3, /* 3 week */
        }
    }
}

impl Config {
    /* Allow the configuration to be extracted from any [`Provider`] */
    #[inline]
    pub fn from<T: Provider>(provider: T) -> Result<Self> {
        Ok(Figment::from(provider).extract()?)
    }

    /** Extract figment configuration from the environment */
    #[inline]
    pub fn figment() -> Figment {
        Figment::from(Config::default()).merge(figment::providers::Env::prefixed("SHREKD_"))
    }

    /** Compute and get the temporary file path */
    #[inline]
    pub fn temp(&self) -> PathBuf {
        self.data_dir.join(TEMPDIR_NAME)
    }

    /** Get the [`RetentionCurve`] from the current configuration */
    #[inline]
    pub fn curve(&self) -> Result<RetentionCurve> {
        RetentionCurve::new(
            self.retention_min_age,
            self.retention_max_age,
            self.max_file_size,
        )
        .map_err(|err| Error::Intrinsics(err.to_string().into()))
    }
}

use figment::value::{Dict, Map};

impl Provider for Config {
    #[inline]
    fn metadata(&self) -> Metadata {
        Metadata::named("shrekd Config")
    }

    #[inline]
    fn data(&self) -> figment::error::Result<Map<Profile, Dict>> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    #[inline]
    fn profile(&self) -> Option<Profile> {
        None
    }
}
