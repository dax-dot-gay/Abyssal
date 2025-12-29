use crate::{
    export_routes,
    models::{GenericUser, Token, User, UserMethods},
    types::Uuid,
    util::Collection,
};
use bson::doc;
use rocket::{post, serde::json::Json};
use rocket_okapi::{JsonSchema, openapi};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
struct LoginResponse {
    pub token: Uuid,
    pub user: GenericUser,
}

#[openapi(tag = "Users")]
#[post("/login", data = "<login>")]
async fn login(
    login: Json<LoginRequest>,
    tokens: Collection<Token>,
    users: Collection<User>,
) -> crate::ApiResult<LoginResponse> {
    if let Some(user) = users
        .find_one(doc! {"name": login.username.clone()})
        .await?
    {
        if user.verify_password(login.password.clone())? {
            let new_token = Token::new(user.id());
            let _ = tokens.save(new_token.clone()).await?;
            Ok(Json(LoginResponse {
                token: new_token.id(),
                user: user.into(),
            }))
        } else {
            Err(crate::Error::IncorrectCredentials)
        }
    } else {
        Err(crate::Error::IncorrectCredentials)
    }
}

export_routes![login];
