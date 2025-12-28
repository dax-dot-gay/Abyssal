mod cli;
mod error;
pub mod models;
pub mod routes;
pub mod types;
pub mod util;

use bson::doc;
use clap::Parser;
pub use error::{Error, ErrorMeta, Result};
use rocket::{Build, Rocket, fairing::AdHoc, launch};
use rocket_okapi::settings::OpenApiSettings;
use spire_enum::prelude::EnumExtensions;

use crate::models::{Model, UserMethods};

async fn launch_inner() -> Rocket<Build> {
    let args = cli::AbyssalCli::parse();
    let config = types::Config::load(args.config_files).expect("Failed to load configuration!");
    let rocket_config = config.rocket_config();
    let (routes, _) = routes::routes(&OpenApiSettings::default());

    rocket::custom(rocket_config)
        .manage(
            mongodb::Client::with_uri_str(config.database().url())
                .await
                .unwrap(),
        )
        .manage(config.clone())
        .mount("/api", routes)
        .attach(AdHoc::on_liftoff("Ensure admin user", |rck| {
            Box::pin(async move {
                let config = rck.state::<types::Config>().unwrap();
                let collection = rck
                    .state::<mongodb::Client>()
                    .unwrap()
                    .database(&config.database().database())
                    .collection::<models::User>(models::User::collection());
                if let Some(existing) = collection
                    .find_one(doc! {"username": config.authentication().admin_user()})
                    .await
                    .unwrap()
                {
                    if !existing.is_var::<models::user::OwnerUser>() {
                        panic!(
                            "Another user with the default administrator's username already exists!"
                        );
                    }
                } else {
                    let created = models::User::create_owner(
                        config.authentication().admin_user(),
                        config.authentication().admin_password(),
                    )
                    .unwrap();
                    collection
                        .replace_one(doc! {"id": created.id()}, created)
                        .upsert(true)
                        .await
                        .unwrap();
                }
            })
        }))
}

#[launch]
async fn launch() -> Rocket<Build> {
    launch_inner().await
}
