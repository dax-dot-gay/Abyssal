use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use rocket::{
    Request,
    http::Status,
    request::{self, FromRequest},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::{models::Model, types::Config};

#[derive(Clone, Debug)]
pub struct Collection<T: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + Model>(
    mongodb::Collection<T>,
);

impl<T: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + Model> Deref
    for Collection<T>
{
    type Target = mongodb::Collection<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + Model> DerefMut
    for Collection<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'r, T: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + Model> FromRequest<'r>
    for Collection<T>
{
    type Error = crate::Error;
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(client) = req.rocket().state::<mongodb::Client>().cloned() {
            if let Some(config) = req.rocket().state::<Config>().cloned() {
                request::Outcome::Success(Self(
                    client
                        .database(&config.database().database())
                        .collection::<T>(T::collection()),
                ))
            } else {
                request::Outcome::Error((
                    Status::InternalServerError,
                    crate::Error::MissingState(String::from("abyssal::Config")),
                ))
            }
        } else {
            request::Outcome::Error((
                Status::InternalServerError,
                crate::Error::MissingState(String::from("mongodb::Client")),
            ))
        }
    }
}
