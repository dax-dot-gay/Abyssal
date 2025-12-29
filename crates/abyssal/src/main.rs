mod cli;
mod error;
pub mod models;
pub mod routes;
pub mod types;
pub mod util;

use bson::doc;
use clap::Parser;
pub use error::{ApiResult, Error, ErrorMeta, Result};
use rocket::{Build, Rocket, fairing::AdHoc, launch};
use rocket_okapi::{
    rapidoc::{ApiConfig, GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc}, settings::{OpenApiSettings, UrlObject}
};
use spire_enum::prelude::EnumExtensions;

async fn launch_inner() -> Rocket<Build> {
    let args = cli::AbyssalCli::parse();
    let config = types::Config::load(args.config_files).expect("Failed to load configuration!");
    let rocket_config = config.rocket_config();
    let (routes, openapi_spec) = routes::routes(&OpenApiSettings::default());

    rocket::custom(rocket_config)
        .manage(
            mongodb::Client::with_uri_str(config.database().url())
                .await
                .unwrap(),
        )
        .manage(config.clone())
        .manage(openapi_spec)
        .mount("/api", routes)
        .mount(
            "/api/doc/openapi",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                api: ApiConfig {
                    server_url: "/api".to_string(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .attach(AdHoc::on_liftoff("Ensure admin user", |rck| {
            Box::pin(async move {
                let config = rck.state::<types::Config>().unwrap();
                let collection = util::Collection::<models::User>::new(
                    rck.state::<mongodb::Client>().unwrap().clone(),
                    config.database().database(),
                );
                if let Some(existing) = collection
                    .find_one(doc! {"name": config.authentication().admin_user()})
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
                    collection.save(created).await.unwrap();
                }
            })
        }))
}

#[launch]
async fn launch() -> Rocket<Build> {
    launch_inner().await
}
