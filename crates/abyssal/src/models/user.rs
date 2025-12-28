use std::str::FromStr;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{Error as PasswordHashError, SaltString, rand_core::{OsRng, RngCore}},
};
use base64::Engine as _;
use bson::doc;
use getset::{CloneGetters, WithSetters};
use rocket::{Request, http::Status, request::{self, FromRequest}};
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use spire_enum::prelude::{delegate_impl, delegated_enum};
use strum::Display;

use crate::{models::Model, types::Uuid, util::Collection};

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

impl ApplicationUser {
    pub(self) fn new(name: String, owner: Uuid, client_id: String, client_secret: String) -> Self {
        Self { id: Uuid::new(), name, groups: vec![], owner, client_id, client_secret }
    }
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

    pub fn create_application(name: impl Into<String>, owner: impl Into<Uuid>) -> crate::Result<(Self, String)> {
        let mut client_id_bytes = [0u8; 32];
        let mut client_secret_bytes = [0u8; 64];

        OsRng.fill_bytes(&mut client_id_bytes);
        OsRng.fill_bytes(&mut client_secret_bytes);

        let client_id = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(client_id_bytes);
        let client_secret = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(client_secret_bytes);
        let hashed_secret = Self::hash_value(client_secret.clone())?;
        Ok((ApplicationUser::new(name.into(), owner.into(), client_id, hashed_secret).into(), client_secret))
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

    pub fn verify_client_secret(&self, secret: impl Into<String>) -> crate::Result<bool> {
        let secret = secret.into();
        let hashed = match self.clone() {
            User::Application(user) => Ok(user.client_secret()),
            _ => Err(crate::Error::invalid_user_type([
                UserKind::Application
            ])),
        }?;

        Self::verify_value(secret, hashed)
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

    async fn from_request_inner(req: &Request<'_>) -> crate::Result<Self> {
        if let Some(authorization) = req.headers().get_one("Authorization").map(|v| v.to_string()) {
            match authorization.split_once(" ") {
                Some(("Token", token)) => {
                    let tokens = Collection::<crate::models::Token>::from_request(req).await.unwrap();
                    let users = Collection::<crate::models::User>::from_request(req).await.unwrap();
                    if let Ok(Some(existing_token)) = tokens.get(Uuid::from_str(token)?).await {
                        if let Ok(Some(existing_user)) = existing_token.resolve_user(users).await {
                            let refreshed = existing_token.refresh_token();
                            let _ = tokens.save(refreshed).await?;
                            Ok(existing_user)
                        } else {
                            let _ = tokens.delete(existing_token.id()).await?;
                            Err(crate::Error::MissingAuthorization)
                        }
                    } else {
                        Err(crate::Error::MissingAuthorization)
                    }
                },
                Some(("Application", app_auth)) => {
                    let users = Collection::<crate::models::User>::from_request(req).await.unwrap();
                    if let Some((client_id, client_secret)) = app_auth.split_once(":") {
                        if let Ok(Some(existing_user)) = users.find_one(doc! {"client_id": client_id}).await {
                            if existing_user.verify_client_secret(client_secret.to_string())? {
                                Ok(existing_user)
                            } else {
                                Err(crate::Error::MissingAuthorization)
                            }
                        } else {
                            Err(crate::Error::MissingAuthorization)
                        }
                    } else {
                        Err(crate::Error::MissingAuthorization)
                    }
                },
                _ => Err(crate::Error::MissingAuthorization)
            }
        } else {
            Err(crate::Error::MissingAuthorization)
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = crate::Error;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match Self::from_request_inner(req).await {
            Ok(resolved) => request::Outcome::Success(resolved),
            Err(err) => request::Outcome::Error((Status::new(err.metadata().status), err)),
        }
    }
}
