use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{Error as PasswordHashError, SaltString, rand_core::OsRng},
};
use getset::{CloneGetters, WithSetters};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use spire_enum::prelude::{delegate_impl, delegated_enum};
use strum::Display;

use crate::{models::Model, types::Uuid};

#[derive(
    Serialize, Deserialize, Clone, Debug, JsonSchema, PartialEq, Eq, PartialOrd, Ord, Hash, Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserKind {
    Local,
    Owner,
    Oidc,
    Application,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
#[getset(get_clone = "pub", set_with = "pub")]
pub struct LocalUser {
    #[serde(default)]
    id: Uuid,
    name: String,
    password: String,

    #[serde(default)]
    groups: Vec<String>,
}

impl LocalUser {
    pub(self) fn new(name: String, password: String) -> Self {
        Self {
            id: Uuid::new(),
            name,
            password,
            groups: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
#[getset(get_clone = "pub", set_with = "pub")]
pub struct OwnerUser {
    #[serde(default)]
    id: Uuid,
    name: String,
    password: String,

    #[serde(default)]
    groups: Vec<String>,
}

impl OwnerUser {
    pub(self) fn new(name: String, password: String) -> Self {
        Self {
            id: Uuid::new(),
            name,
            password,
            groups: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
#[getset(get_clone = "pub", set_with = "pub")]
pub struct OidcUser {
    #[serde(default)]
    id: Uuid,
    name: String,

    #[serde(default)]
    groups: Vec<String>,

    #[serde(default)]
    oidc_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, CloneGetters, WithSetters)]
#[getset(get_clone = "pub", set_with = "pub")]
pub struct ApplicationUser {
    #[serde(default)]
    id: Uuid,
    name: String,

    #[serde(default)]
    groups: Vec<String>,
    owner: Uuid,
    client_id: String,
    client_secret: String,
}

pub trait UserMethods {
    fn id(&self) -> Uuid;
    fn name(&self) -> String;
    fn groups(&self) -> Vec<String>;

    fn with_id(self, id: Uuid) -> Self;
    fn with_name(self, name: String) -> Self;
    fn with_groups(self, groups: Vec<String>) -> Self;

    fn with_group(self, group: impl Into<String>) -> Self;

    fn without_group(self, group: impl Into<String>) -> Self;
}

#[delegated_enum(impl_conversions)]
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum User {
    Local(LocalUser),
    Owner(OwnerUser),
    Oidc(OidcUser),
    Application(ApplicationUser),
}

#[delegate_impl]
impl UserMethods for User {
    fn id(&self) -> Uuid;
    fn name(&self) -> String;
    fn groups(&self) -> Vec<String>;
    fn with_id(self, id: Uuid) -> Self;
    fn with_name(self, name: String) -> Self;
    fn with_groups(self, groups: Vec<String>) -> Self;

    fn with_group(self, group: impl Into<String>) -> Self {
        let mut current_groups = self.groups();
        current_groups.push(group.into());
        self.with_groups(current_groups)
    }

    fn without_group(self, group: impl Into<String>) -> Self {
        let group = group.into();
        let current_groups = self.groups();
        self.with_groups(
            current_groups
                .into_iter()
                .filter(|v| v.clone() != group)
                .collect(),
        )
    }
}

impl Model for User {
    fn collection() -> &'static str {
        "auth.users"
    }

    fn model_id(&self) -> Uuid {
        self.id()
    }
}

impl User {
    pub fn kind(&self) -> UserKind {
        match self.clone() {
            User::Local(..) => UserKind::Local,
            User::Owner(..) => UserKind::Owner,
            User::Oidc(..) => UserKind::Oidc,
            User::Application(..) => UserKind::Application,
        }
    }

    fn hash_value(value: impl Into<String>) -> crate::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2
            .hash_password(value.into().as_bytes(), &salt)?
            .to_string())
    }

    fn verify_value(value: impl Into<String>, hashed: impl Into<String>) -> crate::Result<bool> {
        let value = value.into();
        let hashed = hashed.into();
        let parsed_hash = PasswordHash::new(hashed.as_str())?;
        match Argon2::default().verify_password(value.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(PasswordHashError::Password) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_local(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> crate::Result<Self> {
        let username = username.into();
        let password = password.into();
        let hashed_password = Self::hash_value(password)?;
        Ok(LocalUser::new(username, hashed_password).into())
    }

    pub fn create_owner(
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> crate::Result<Self> {
        let username = username.into();
        let password = password.into();
        let hashed_password = Self::hash_value(password)?;
        Ok(OwnerUser::new(username, hashed_password).into())
    }

    pub fn verify_password(&self, password: impl Into<String>) -> crate::Result<bool> {
        let password = password.into();
        let hashed = match self.clone() {
            User::Local(user) => Ok(user.password()),
            User::Owner(user) => Ok(user.password()),
            _ => Err(crate::Error::invalid_user_type([
                UserKind::Local,
                UserKind::Owner,
            ])),
        }?;

        Self::verify_value(password, hashed)
    }

    pub fn with_password(self, new_password: impl Into<String>) -> crate::Result<Self> {
        let hashed_password = Self::hash_value(new_password)?;
        match self {
            User::Local(user) => Ok(user.with_password(hashed_password).into()),
            User::Owner(user) => Ok(user.with_password(hashed_password).into()),
            _ => Err(crate::Error::invalid_user_type([
                UserKind::Local,
                UserKind::Owner,
            ])),
        }
    }
}
