use okapi::openapi3::OpenApi;
use rocket::{Build, Rocket};
use rocket_okapi::{
    get_nested_endpoints_and_docs, mount_endpoints_and_merged_docs, settings::OpenApiSettings,
};

mod misc;

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    get_nested_endpoints_and_docs! {
        "/" => misc::routes(settings)
    }
}

#[macro_export]
macro_rules! export_routes {
    [$($routes:ident),+] => {
        pub fn routes(settings: &rocket_okapi::settings::OpenApiSettings) -> (Vec<rocket::Route>, okapi::openapi3::OpenApi) {
            rocket_okapi::openapi_get_routes_spec![settings: $($routes).+]
        }
    };
}
