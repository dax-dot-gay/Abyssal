mod cli;
mod error;
pub mod models;
pub mod routes;
pub mod types;
pub mod util;
pub use types::Config;

use bson::doc;
use clap::Parser;
pub use error::{ApiResult, Error, ErrorMeta, Result};
use rocket::{Build, Rocket, fairing::AdHoc, launch};
use rocket_okapi::{
    rapidoc::{ApiConfig, GeneralConfig, HideShowConfig, RapiDocConfig, make_rapidoc},
    settings::{OpenApiSettings, UrlObject},
};

use crate::models::UserMethods;

async fn launch_inner() -> Rocket<Build> {
    let args = cli::AbyssalCli::parse();
    let config =
        types::Config::load(args.config_files.clone()).expect("Failed to load configuration!");
    let rocket_config = config.rocket_config();
    let (routes, openapi_spec) = routes::routes(&OpenApiSettings::default());

    if let Some(spec_path) = args.specification_path.clone() {
        let serialized = serde_json::to_string_pretty(&openapi_spec).unwrap();
        if let Some(parent) = spec_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        std::fs::write(spec_path, serialized).unwrap();
    }

    if !config.filesystem().filesystem().is_dir() {
        std::fs::create_dir_all(config.filesystem().filesystem())
            .expect("Should be able to create the filesystem root");
    }

    if !config.filesystem().filesystem().join(".abyssal").is_dir() {
        std::fs::create_dir_all(config.filesystem().filesystem().join(".abyssal"))
            .expect("Should be able to create the .abyssal directory");
    }

    rocket::custom(rocket_config)
        .manage(
            mongodb::Client::with_uri_str(config.database().url())
                .await
                .unwrap(),
        )
        .manage(config.clone())
        .manage(
            sled::Config::default()
                .mode(sled::Mode::HighThroughput)
                .use_compression(true)
                .path(config.filesystem().filesystem().join(".abyssal/meta.db"))
                .open()
                .expect("Should be able to open/create meta.db")
        )
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
                    if !existing.permissions().has_permission(types::Permission::Administrator) {
                        panic!(
                            "Another user with the default administrator's username already exists and is not an admin!"
                        );
                    }
                } else {
                    let created = models::User::create_local(
                        config.authentication().admin_user(),
                        config.authentication().admin_password(),
                    )
                    .unwrap();
                    created.permissions().set_permission(types::Permission::Administrator);
                    collection.save(created).await.unwrap();
                }
            })
        }))
        .attach(util::generate_resources())
}

#[launch]
async fn launch() -> Rocket<Build> {
    launch_inner().await
}
