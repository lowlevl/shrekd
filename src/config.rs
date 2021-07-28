use figment::{Figment, Metadata, Profile, Provider};
use rocket::data::ToByteUnit;
use serde::{Deserialize, Serialize};

use std::path::{PathBuf};

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
    /** Random URI's slug length */
    pub slug_length: u8,

    /** Max file size */
    pub max_file_size: u64,
    /** Max paste size */
    pub max_paste_size: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: String::from("0.0.0.0"),
            port: 8000,
            redis_url: String::from("redis://127.0.0.1:6379"),
            data_dir: PathBuf::from("/tmp/.shrtd"),
            slug_length: 13,
            max_file_size: 128.megabytes().into(),
            max_paste_size: 1.megabytes().into(),
        }
    }
}

impl Config {
    /* Allow the configuration to be extracted from any [`Provider`] */
    pub fn from<T: Provider>(provider: T) -> Result<Self, crate::Error> {
        Ok(Figment::from(provider).extract()?)
    }

    /** Extract figment configuration from the environment */
    pub fn figment() -> Figment {
        Figment::from(Config::default()).merge(figment::providers::Env::prefixed("SHRTD_"))
    }

    /** Compute and get the temporary file path */
    pub fn temp(&self) -> PathBuf {
        self.data_dir.join(TEMPDIR_NAME)
    }
}

use figment::value::{Dict, Map};

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("shrtd Config")
    }

    fn data(&self) -> figment::error::Result<Map<Profile, Dict>> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}
