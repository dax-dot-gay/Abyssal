
use std::path::PathBuf;

use okapi::openapi3::OpenApi;
use rocket::{State, get, serde::json::Json};
use rocket_okapi::{JsonSchema, openapi};
use serde::{Deserialize, Serialize};

use crate::{export_routes, models::{Permissions, PermissionsDescription, permission::PermissionsMethods}};

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

#[openapi]
#[get("/info/permissions")]
async fn get_info_permissions() -> Json<PermissionsDescription> {
    Json(Permissions::describe())
}

#[openapi]
#[get("/info/permissions/<path..>")]
async fn get_info_permissions_at_path(path: PathBuf) -> Json<Option<PermissionsDescription>> {
    let path = path.to_string_lossy().to_string();
    Json(Permissions::describe().at_path(path))
}

#[openapi(skip)]
#[get("/doc/openapi.json")]
async fn get_openapi_json(spec: &State<OpenApi>) -> Json<OpenApi> {
    Json(spec.inner().clone())
}

export_routes![get_info, get_openapi_json, get_info_permissions, get_info_permissions_at_path];
