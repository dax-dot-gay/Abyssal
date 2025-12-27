mod cli;
mod error;
pub mod models;
pub mod routes;
pub mod types;
pub mod util;

use clap::Parser;
pub use error::{Error, ErrorMeta, Result};
use rocket::{Build, Rocket, fairing::AdHoc, launch};
use rocket_db_pools::Database;
use rocket_okapi::settings::OpenApiSettings;

fn launch_inner() -> Rocket<Build> {
    let args = cli::AbyssalCli::parse();
    let config = types::Config::load(args.config_files).expect("Failed to load configuration!");
    let rocket_config = config.rocket_config();
    let (routes, _) = routes::routes(&OpenApiSettings::default());

    rocket::custom(rocket_config)
        .attach(util::AbyssalDb::init())
        .manage(config.clone())
        .mount("/api", routes)
        .attach(AdHoc::on_liftoff("Ensure admin user", |rck| Box::pin(async move {
            let db = rck.state::<util::AbyssalDb>().unwrap().db();
            let config = rck.state::<types::Config>().unwrap();
            if let Some(existing) = models::LocalUser::select_one_with_username(&db, config.authentication().admin_user()).await.unwrap() {
                if !existing.default_admin() {
                    panic!("Another user with the default administrator's username already exists!");
                }
            } else {
                let created = models::LocalUser::new_default_admin(config.authentication().admin_user(), config.authentication().admin_password()).unwrap();
                created.save(&db).await.unwrap();
            }
        })))
}

#[launch]
fn launch() -> Rocket<Build> {
    launch_inner()
}
