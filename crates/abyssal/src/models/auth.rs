use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash::{PasswordHasher, SaltString, rand_core::OsRng, Error as PasswordHashError}};
use getset::{CloneGetters, Setters};
use rbatis::rbdc::Uuid;
use rbatis_derive::Schema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Schema, CloneGetters, Setters)]
#[schema(table(name = "local_users"))]
#[getset(get_clone = "pub")]
pub struct LocalUser {
    #[field(select)]
    id: Uuid,

    #[field(unique, select)]
    #[getset(set = "pub")]
    username: String,

    #[getset(skip)]
    password: String,

    #[serde(default)]
    #[getset(set = "pub")]
    groups: Vec<Uuid>
}

impl LocalUser {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> crate::Result<Self> {
        let username = username.into();
        let password = password.into();
        Ok(Self { id: Uuid::new(), username, password: Self::make_password(password)?, groups: vec![] })
    }

    fn make_password(password: impl Into<String>) -> crate::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        Ok(argon2.hash_password(password.into().as_bytes(), &salt)?.to_string())
    }

    pub fn verify(&self, password: impl Into<String>) -> crate::Result<()> {
        let parsed_hash = PasswordHash::new(&self.password.as_str())?;
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
