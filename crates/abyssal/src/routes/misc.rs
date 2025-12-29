
use okapi::openapi3::OpenApi;
use rocket::{State, get, serde::json::Json};
use rocket_okapi::{JsonSchema, openapi};
use serde::{Deserialize, Serialize};

use crate::export_routes;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
struct GetInfoResponse {
    pub version: String,
}

#[openapi]
#[get("/info")]
async fn get_info() -> Json<GetInfoResponse> {
    Json(GetInfoResponse {
        version: clap::crate_version!().to_string(),
    })
}

#[openapi(skip)]
#[get("/doc/openapi.json")]
async fn get_openapi_json(spec: &State<OpenApi>) -> Json<OpenApi> {
    Json(spec.inner().clone())
}

export_routes![get_info, get_openapi_json];
