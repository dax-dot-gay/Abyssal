#[abyssal_macros::make_error]
pub enum Error {
    #[error(format = "An unknown error occurred: {0:?}", arc, from)]
    Unknown(anyhow::Error),

    #[error(format = "Configuration error: {0:?}", arc, from)]
    Configuration(figment::Error),

    #[error(format = "ORM error: {0:?}", arc)]
    SeaOrm(SeaOrmError)
}

#[derive(Debug, thiserror::Error)]
pub enum SeaOrmError {
    #[error(transparent)]
    Db(#[from] sea_orm::error::DbErr),
    #[error(transparent)]
    Runtime(#[from] sea_orm::error::RuntimeErr),
    #[error(transparent)]
    Sql(#[from] sea_orm::error::SqlErr),
    #[error(transparent)]
    Sqlx(#[from] sea_orm::error::SqlxError),
    #[error(transparent)]
    MySql(#[from] sea_orm::error::SqlxMySqlError),
    #[error(transparent)]
    Sqlite(#[from] sea_orm::error::SqlxSqliteError),
    #[error(transparent)]
    ColumnFromStr(#[from] sea_orm::error::ColumnFromStrErr),
    #[error(transparent)]
    TryGet(#[from] sea_orm::error::TryGetError),
}

impl<T: Into<SeaOrmError>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::SeaOrm(Arc::new(value.into()))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
