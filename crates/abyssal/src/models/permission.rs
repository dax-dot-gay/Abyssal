use abyssal_macros::make_permissions;
use getset::{CloneGetters, WithSetters};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionTarget {
    Group,
    User,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
pub struct PermissionModel {
    #[serde(default)]
    id: Uuid,

    target_type: PermissionTarget,
    target_id: Uuid,
}

make_permissions! {
    Permissions => {
        Authentication => {
            Users => {
                View "Only allows viewing user configurations";
            } "Allows modification of user configurations";
            Oidc => {
                View "Only allows viewing OIDC provider information";
            } "Allows modification of OIDC provider(s)";
            Applications => {
                Owned "Only allows management of owned applications";
                Others => "application" {
                    View "Only allows viewing this application";
                } "Allows management of other applications (app name in <application>)";
            } "Allows modification of API clients";
        };
    }
}
