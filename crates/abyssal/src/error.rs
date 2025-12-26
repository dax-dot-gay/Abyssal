#[abyssal_macros::make_error]
pub enum Error {
    #[error(format = "An unknown error occurred: {0:?}", arc, from)]
    Unknown(anyhow::Error),

    #[error(format = "Configuration error: {0:?}", arc, from)]
    Configuration(figment::Error),

    #[error(format = "ORM error: {0:?}", arc, from)]
    Orm(rbatis::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
