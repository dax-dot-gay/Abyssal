use rbatis::rbdc::Uuid;
use rbatis_derive::Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Schema)]
#[schema(table(name = "auth_user"))]
pub struct AuthUser {
    pub id: Uuid,

    #[field(unique)]
    pub username: String
}
