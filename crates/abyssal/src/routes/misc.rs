
use rocket::{get, serde::json::Json};
use rocket_okapi::{JsonSchema, openapi};
use serde::{Deserialize, Serialize};

use crate::export_routes;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
struct GetInfoResponse {
    pub version: String,
}

#[openapi(tag = "Misc")]
#[get("/info")]
async fn get_info() -> Json<GetInfoResponse> {
    Json(GetInfoResponse {
        version: clap::crate_version!().to_string(),
    })
}

export_routes![get_info];
