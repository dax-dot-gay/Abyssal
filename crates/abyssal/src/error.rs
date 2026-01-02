use std::fmt::Display;

use argon2::password_hash;
use rocket::serde::json::Json;

#[abyssal_macros::make_error]
pub enum Error {
    #[error(format = "An unknown error occurred: {0:?}", arc, from, code = "server.unknown")]
    Unknown(anyhow::Error),

    #[error(format = "Configuration error: {0:?}", arc, from, code = "server.configuration")]
    Configuration(figment::Error),

    #[error(format = "Database error: {0:?}", arc, from, code = "server.database")]
    Database(mongodb::error::Error),

    #[error(format = "Password generation error: {0:?}", arc, from, code = "server.password_generation")]
    PasswordGeneration(password_hash::Error),

    #[error(format = "Invalid credentials (incorrect username/password)", status = 401, code = "auth.credentials")]
    IncorrectCredentials,

    #[error(format = "Missing application state (critical): <{0}>", code = "server.missing_state")]
    MissingState(String),

    #[error(format = "Invalid user type: should be one of [{0}]", code = "auth.invalid_user_type")]
    InvalidUserType(String),

    #[error(format = "Missing/invalid authorization header or cookie", code = "auth.missing", status = 401)]
    MissingAuthorization,

    #[error(format = "UUID encoding/decoding error: {0:?}", arc, from, code = "server.uuid")]
    Uuid(uuid::Error),

    #[error(format = "Unknown permission string: {0}", code = "server.bad_permission")]
    UnknownPermission(String),

    #[error(format = "IO error: {0:?}", arc, from, code = "server.io")]
    Io(std::io::Error)
}

impl Error {
    pub fn invalid_user_type(expected: impl IntoIterator<Item = impl Display>) -> Self {
        let array = expected.into_iter().map(|v| v.to_string()).collect::<Vec<_>>();
        Self::InvalidUserType(array.join(", "))
    }

    pub fn unknown_permission(permission: impl Into<String>) -> Self {
        Self::UnknownPermission(permission.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
pub type ApiResult<T> = std::result::Result<Json<T>, Error>;
