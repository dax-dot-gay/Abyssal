pub mod user;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
pub use user::{User, UserKind, UserMethods};

use crate::types::Uuid;

pub trait Model: Serialize + DeserializeOwned + Clone + Debug + Send + Sync {
    fn collection() -> &'static str;
    fn model_id(&self) -> Uuid;
    fn model_id_field() -> String {
        "id".to_string()
    }
}