use std::{fmt::Display, str::FromStr};

use bson::Bson;
use serde::{Deserialize, Serialize};
use uuid::Uuid as RawUuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, rocket_okapi::JsonSchema)]
pub struct Uuid(String);

impl Uuid {
    pub fn new() -> Self {
        Self(RawUuid::new_v4().to_string())
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Default for Uuid {
    fn default() -> Self {
        Self::new()
    }
}

impl From<RawUuid> for Uuid {
    fn from(value: RawUuid) -> Self {
        Self(value.to_string())
    }
}

impl From<Uuid> for RawUuid {
    fn from(value: Uuid) -> Self {
        RawUuid::from_str(&value.to_string()).unwrap()
    }
}

impl FromStr for Uuid {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(RawUuid::from_str(s)?))
    }
}

impl From<Uuid> for Bson {
    fn from(value: Uuid) -> Self {
        Bson::String(value.to_string())
    }
}
