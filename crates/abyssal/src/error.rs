use argon2::password_hash;

#[abyssal_macros::make_error]
pub enum Error {
    #[error(format = "An unknown error occurred: {0:?}", arc, from, code = "server.unknown")]
    Unknown(anyhow::Error),

    #[error(format = "Configuration error: {0:?}", arc, from, code = "server.configuration")]
    Configuration(figment::Error),

    #[error(format = "ORM error: {0:?}", arc, from, code = "server.orm")]
    Orm(rbatis::Error),

    #[error(format = "Password generation error: {0:?}", arc, from, code = "server.password_generation")]
    PasswordGeneration(password_hash::Error),

    #[error(format = "Invalid credentials (incorrect username/password)", status = 403, code = "auth.credentials")]
    IncorrectCredentials
}

pub type Result<T> = std::result::Result<T, Error>;
