/*!
 * # shrt
 * SHare and shoRTen, simple file & url sharing service.
 */

use figment::Figment;
use simplelog::{ColorChoice, Config as SimpleLogConfig, LevelFilter, TermLogger, TerminalMode};
use std::fs;

mod config;
mod routes;
mod types;

pub use config::Config;
pub use types::{Error, Result};

#[rocket::launch]
fn rocket() -> _ {
    let config = Config::from(Config::figment()).unwrap();

    /* Initialize the logger with it's colors and filters */
    TermLogger::init(
        LevelFilter::Trace,
        SimpleLogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    log::info!(
        "Initiating the launch of the `rocket` server on `{}:{}` !",
        config.address,
        config.port
    );

    /* Initialize the directories needed for data storage */
    fs::create_dir_all(&config.tmp_dir).unwrap();
    fs::create_dir_all(&config.data_dir).unwrap();

    /* Configure and launch `rocket` from the `Config` structure */
    rocket::custom(
        Figment::from(rocket::Config::default())
            .merge(("address", &config.address))
            .merge(("port", &config.port))
            .merge(("temp_dir", &config.tmp_dir))
            .merge(("limits.file", &config.max_file_size))
            .merge(("limits.bytes", &config.max_paste_size)),
    )
    /* Mount `/` routes */
    .mount("/", routes::mounts())
    /* Attach a redis client to the rocket instance */
    .manage(redis::Client::open(config.redis_url.clone()).unwrap())
    /* Attach the config to the rocket instance */
    .manage(config)
}
