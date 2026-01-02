use std::path::{Path, PathBuf};

use bson::doc;
use getset::{CloneGetters, WithSetters};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{models::Model, types::Uuid, util::Collection};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
#[getset(get_clone = "pub", set_with = "pub")]
pub struct RootDirectory {
    #[serde(default)]
    id: Uuid,

    name: String,

    #[serde(default)]
    display_name: Option<String>,

    path: PathBuf,
}

impl Model for RootDirectory {
    fn collection() -> &'static str {
        "resources.roots"
    }

    fn model_id(&self) -> Uuid {
        self.id()
    }
}

impl RootDirectory {
    pub fn new(
        name: impl Into<String>,
        display_name: Option<impl Into<String>>,
        path: impl AsRef<Path>,
    ) -> Self {
        Self {
            id: Uuid::new(),
            name: name.into(),
            display_name: display_name.and_then(|v| Some(v.into())),
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[rocket::async_trait]
pub trait RootDirectoryCollectionExt {
    async fn by_name(
        &self,
        name: impl Into<String> + Send + Sync,
    ) -> crate::Result<Option<RootDirectory>>;
}

#[rocket::async_trait]
impl RootDirectoryCollectionExt for Collection<RootDirectory> {
    async fn by_name(
        &self,
        name: impl Into<String> + Send + Sync,
    ) -> crate::Result<Option<RootDirectory>> {
        Ok(self.find_one(doc! {"name": name.into()}).await?)
    }
}
