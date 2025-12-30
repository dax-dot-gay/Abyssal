use crate::types::Uuid;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub trait Model: Serialize + DeserializeOwned + Clone + Debug + Send + Sync {
    fn collection() -> &'static str;
    fn model_id(&self) -> Uuid;
    fn model_id_field() -> String {
        "id".to_string()
    }
}

pub mod user;
pub use user::{User, UserKind, UserMethods, GenericUser};

pub mod token;
pub use token::Token;

pub mod permission;
pub use permission::{Permissions, PermissionsDescription};