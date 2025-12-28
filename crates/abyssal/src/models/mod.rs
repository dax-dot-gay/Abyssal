pub mod auth;

pub use auth::{LocalUser, Session};

pub trait Model {
    fn collection() -> &'static str;
}