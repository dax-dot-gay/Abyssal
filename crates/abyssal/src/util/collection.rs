use std::{
    ops::{Deref, DerefMut},
};

use bson::doc;
use rocket::{
    Request,
    http::Status,
    request::{self, FromRequest},
};
use rocket_okapi::{r#gen::OpenApiGenerator, request::{OpenApiFromRequest, RequestHeaderInput}};

use crate::{models::Model, types::{Config, Uuid}};

#[derive(Clone, Debug)]
pub struct Collection<T: Model>(
    mongodb::Collection<T>,
);

impl<T: Model> Deref
    for Collection<T>
{
    type Target = mongodb::Collection<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Model> DerefMut
    for Collection<T>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rocket::async_trait]
impl<'r, T: Model> FromRequest<'r>
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

impl<'r, T: Model> OpenApiFromRequest<'r> for Collection<T> {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

impl<T: Model> Collection<T> {
    pub fn new(client: mongodb::Client, database: impl Into<String>) -> Self {
        Self(client.database(database.into().as_str()).collection::<T>(T::collection()))
    }

    pub async fn get(&self, id: impl Into<Uuid>) -> crate::Result<Option<T>> {
        Ok(self.find_one(doc! {T::model_id_field(): id.into()}).await?)
    }

    pub async fn save(&self, model: T) -> crate::Result<Option<T>> {
        Ok(self.find_one_and_replace(doc! {T::model_id_field(): model.model_id()}, model).upsert(true).await?)
    }

    pub async fn delete(&self, id: impl Into<Uuid>) -> crate::Result<Option<T>> {
        Ok(self.find_one_and_delete(doc! {T::model_id_field(): id.into()}).await?)
    }
}
