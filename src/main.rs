/*!
 * # shrtd
 * SHare and shoRTen Daemon, simple file & url sharing service.
 */
#![allow(clippy::large_enum_variant)] /* <- This allows for storing the `rocket::response::Redirect` type inside enums, because these are HUGE */

use figment::Figment;
use simplelog::{ColorChoice, Config as SimpleLogConfig, LevelFilter, TermLogger, TerminalMode};
use tokio::fs;

mod config;
mod routes;
mod types;

pub use config::Config;
pub use types::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Load application configuration from the default provider */
    let config = Config::from(Config::figment()).expect("Failed to load configuration");

    /* Initialize the logger with it's colors and filters */
    TermLogger::init(
        LevelFilter::Trace,
        SimpleLogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to initialize the logger");

    log::info!(
        "Creating the permanent ({:?}) & temporary ({:?}) data directories",
        config.data_dir,
        config.temp(),
    );

    /* Initialize the directories needed for data storage */
    fs::create_dir_all(&config.data_dir)
        .await
        .expect("Failed to create the permanent data directory");
    fs::create_dir_all(&config.temp())
        .await
        .expect("Failed to create the temporary data directory");

    log::info!("Initializing the Redis client with {}", config.redis_url);

    /* Instanciate the Redis client */
    let redis = redis::Client::open(config.redis_url.as_str())
        .expect("Failed to initialize the Redis client");

    /* Get the rocket instance from the configuration */
    let rocket = rocket(config.clone(), redis.clone())
        .ignite()
        .await
        .expect("Failed to ignite the `Rocket` instance");

    log::info!(
        "Finally launching the server on `{}:{}` !",
        config.address,
        config.port
    );

    /* Macro launches concurently two expressions and resumes when one finishes */
    tokio::select! {
        /* This launches the cleanup handler */
        res = cleanup(config.clone(), redis) => {
            Ok(res?)
        },
        /* This launches the server */
        res = rocket.launch() => {
            Ok(res?)
        }
    }
}

async fn cleanup(config: Config, redis: redis::Client) -> crate::Result<()> {
    /*! Listen for `del` and `expired` Redis keyspace events to cleanup expired files */
    use futures::StreamExt;
    use std::path::Path;

    let mut conn = redis.get_async_connection().await?;

    /* Enable keyspace events in the redis server */
    redis::cmd("CONFIG")
        .arg("SET")
        .arg("notify-keyspace-events")
        .arg("Egx") /* `Egx` means E: keyevent events, with types g: general and x: expired */
        .query_async(&mut conn)
        .await?;

    /* Subscribe to the relevant events */
    let mut pubsub = redis.get_async_connection().await?.into_pubsub();
    pubsub.psubscribe("__keyevent@0__:expired").await?;
    pubsub.psubscribe("__keyevent@0__:del").await?;

    let mut events = pubsub.into_on_message();

    loop {
        use types::Record;

        let msg = match events.next().await {
            None => continue,
            Some(msg) => msg,
        };
        log::trace!("Received a new notification: {:#?}", msg);

        /* Retrieve the key, and split it into prefix and slug */
        let mut key: String = msg.get_payload()?;
        let (slug, prefix) = (key.split_off(types::STORAGE_PREFIX.len()), key);

        /* Check if the prefix is good and that the key hasn't been re-created */
        if prefix != types::STORAGE_PREFIX || Record::exists(&slug, &mut conn).await? {
            continue;
        }

        /* Removing the file if needed and found */
        match fs::canonicalize(Path::new(&config.data_dir).join(&slug)).await {
            /* File exists, we remove it */
            Ok(path) => {
                log::debug!("Removing the {:?} since it's record expired", path);
                fs::remove_file(path).await?
            }
            /* Otherwise we skip the notification */
            _ => log::debug!("File was not found, so we have nothing to remove"),
        };
    }
}

fn rocket(config: Config, redis: redis::Client) -> rocket::Rocket<rocket::Build> {
    /*! Configure the [`Rocket`] from the [`Config`] structure, and attach everything */
    rocket::custom(
        Figment::from(rocket::Config::default())
            .merge(("address", &config.address))
            .merge(("port", &config.port))
            .merge(("temp_dir", &config.temp()))
            .merge(("limits.file", &config.max_file_size))
            .merge(("limits.bytes", &config.max_paste_size)),
    )
    /* Mount `/` routes */
    .mount("/", routes::mounts())
    /* Attach a redis client to the rocket instance */
    .manage(redis)
    /* Attach the config to the rocket instance */
    .manage(config)
}
