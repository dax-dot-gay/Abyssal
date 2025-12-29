use okapi::openapi3::OpenApi;
use rocket_okapi::{
    get_nested_endpoints_and_docs, settings::OpenApiSettings,
};

mod misc;
mod users;

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    get_nested_endpoints_and_docs! {
        "/" => misc::routes(settings),
        "/users" => users::routes(settings)
    }
}

#[macro_export]
macro_rules! export_routes {
    [$($routes:ident),+] => {
        pub fn routes(settings: &rocket_okapi::settings::OpenApiSettings) -> (Vec<rocket::Route>, okapi::openapi3::OpenApi) {
            rocket_okapi::openapi_get_routes_spec![settings: $($routes),+]
        }
    };
}
