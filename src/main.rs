/*!
 * # shrt
 * SHare and shoRTen, simple file & url sharing service.
 */

use figment::Figment;
use simplelog::{ColorChoice, Config as SimpleLogConfig, LevelFilter, TermLogger, TerminalMode};
use std::fs;

mod config;
mod error;
mod routes;

pub use config::Config;
pub use error::{Error, Result};

#[rocket::launch]
fn rocket() -> _ {
    let config = Config::from(
        Figment::from(Config::default()).merge(figment::providers::Env::prefixed("SHRT_")),
    )
    .unwrap();

    /* Initialize the logger with it's colors and filters */
    TermLogger::init(
        LevelFilter::Trace,
        SimpleLogConfig::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    log::info!("Initiating the launch of the `rocket` server !");

    /* Initialize the directories needed for data */
    fs::create_dir_all(&config.tmp_dir).unwrap();
    fs::create_dir_all(&config.data_dir).unwrap();

    /* Configure and launch `rocket` from the `Config` structure */
    rocket::custom(
        Figment::from(rocket::Config::default())
            .merge(("address", &config.address))
            .merge(("port", &config.port))
            .merge(("temp_dir", &config.tmp_dir)),
    )
    .mount("/", routes::mounts())
    .manage(config)
}
