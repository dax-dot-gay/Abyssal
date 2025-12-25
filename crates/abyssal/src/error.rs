use std::sync::Arc;

macro_rules! from {
    ($target:ident, $source:path) => {
        impl From<$source> for Error {
            fn from(value: $source) -> Self {
                Self::$target(Arc::new(value))
            }
        }
    };
}

#[abyssal_macros::make_error]
pub enum Error {
    #[error(format = "{0:?}")]
    Unknown(Arc<anyhow::Error>),
}

from!(Unknown, anyhow::Error);

pub type Result<T> = std::result::Result<T, Error>;
