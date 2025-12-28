use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::{PasswordHasher, SaltString, rand_core::OsRng, Error as PasswordHashError}};
use chrono::{DateTime, Utc};
use getset::{CloneGetters, Setters};
use serde::{Deserialize, Serialize};
use crate::{models::Model, types::Uuid};

#[derive(Clone, Debug, Serialize, Deserialize, CloneGetters, Setters)]
#[getset(get_clone = "pub")]
pub struct LocalUser {
    #[serde(default)]
    id: Uuid,

    #[getset(set = "pub")]
    username: String,

    #[getset(skip)]
    password: String,

    #[serde(default)]
    #[getset(set = "pub")]
    groups: Vec<Uuid>,

    #[serde(default)]
    default_admin: bool
}

impl Model for LocalUser {
    fn collection() -> &'static str {
        "auth.users.local"
    }
}

impl LocalUser {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> crate::Result<Self> {
        let username = username.into();
        let password = password.into();
        Ok(Self { id: Uuid::new(), username, password: Self::make_password(password)?, groups: vec![], default_admin: false })
    }

    pub fn new_default_admin(username: impl Into<String>, password: impl Into<String>) -> crate::Result<Self> {
        let username = username.into();
        let password = password.into();
        Ok(Self { id: Uuid::new(), username, password: Self::make_password(password)?, groups: vec![], default_admin: true })
    }

    fn make_password(password: impl Into<String>) -> crate::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.into().as_bytes(), &salt)?.to_string())
    }

    pub fn verify(&self, password: impl Into<String>) -> crate::Result<()> {
        let parsed_hash = PasswordHash::new(self.password.as_str())?;
        match Argon2::default().verify_password(password.into().as_bytes(), &parsed_hash) {
            Ok(_) => Ok(()),
            Err(PasswordHashError::Password) => Err(crate::Error::IncorrectCredentials),
            Err(e) => Err(e.into())
        }
    }

    pub fn set_password(&mut self, current_password: impl Into<String>, new_password: impl Into<String>) -> crate::Result<()> {
        self.verify(current_password)?;
        self.password = Self::make_password(new_password)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, CloneGetters, Setters)]
#[getset(get_clone = "pub")]
pub struct Session {
    #[serde(default)]
    id: Uuid,

    #[getset(set = "pub")]
    #[serde(default)]
    user: Option<Uuid>,

    created: DateTime<Utc>,
    
    #[getset(set = "pub")]
    accessed: DateTime<Utc>
}

impl Model for Session {
    fn collection() -> &'static str {
        "auth.session"
    }
}
