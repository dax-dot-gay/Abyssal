pub mod types;
pub mod util;
pub mod routes;
mod error;
mod cli;

use clap::Parser;
pub use error::{Error, ErrorMeta, Result};
use rocket::{Build, Rocket, launch};
use rocket_db_pools::Database as _;
use rocket_okapi::settings::OpenApiSettings;

async fn launch_inner() -> Rocket<Build> {
    let args = cli::AbyssalCli::parse();
    let config = types::Config::load(args.config_files).expect("Failed to load configuration!");
    let rocket_config = config.rocket_config();
    let (routes, openapi) = routes::routes(&OpenApiSettings::default());

    rocket::custom(rocket_config).attach(util::ORM::init()).manage(config.clone()).mount("/api", routes)
}

#[launch]
async fn launch() -> Rocket<Build> {
    launch_inner()
}
