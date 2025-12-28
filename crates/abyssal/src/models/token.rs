use chrono::{DateTime, Utc};
use getset::CloneGetters;
use serde::{Deserialize, Serialize};

use crate::{
    models::{Model, User},
    types::Uuid,
};

#[derive(Serialize, Deserialize, Clone, Debug, rocket_okapi::JsonSchema, CloneGetters)]
#[getset(get_clone = "pub")]
pub struct Token {
    #[serde(default)]
    id: Uuid,

    user: Uuid,
    created: DateTime<Utc>,
    refreshed: DateTime<Utc>,
}

impl Model for Token {
    fn collection() -> &'static str {
        "auth.tokens"
    }

    fn model_id(&self) -> Uuid {
        self.id()
    }
}

impl Token {
    pub fn new(user: impl Into<Uuid>) -> Self {
        Self {
            id: Uuid::new(),
            user: user.into(),
            created: Utc::now(),
            refreshed: Utc::now(),
        }
    }

    pub fn refresh_token(mut self) -> Self {
        self.refreshed = Utc::now();
        self
    }

    pub async fn resolve_user(
        &self,
        collection: crate::util::Collection<User>,
    ) -> crate::Result<Option<User>> {
        collection.get(self.user()).await
    }
}
