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
                Read;
            };
            Oidc => {
                Read;
            };
            Applications => {
                Owned;
                Others;
            };
        };
    }
}
