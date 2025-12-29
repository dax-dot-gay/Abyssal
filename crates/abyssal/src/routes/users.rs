use crate::{
    export_routes,
    models::{GenericUser, Token, User, UserMethods},
    types::Uuid,
    util::Collection,
};
use bson::doc;
use rocket::{get, post, serde::json::Json};
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

#[openapi(tag = "Users")]
#[post("/logout")]
async fn logout(user: User, tokens: Collection<Token>) -> crate::Result<()> {
    if let Some(existing) = tokens.find_one(doc! {"user": user.id()}).await? {
        let _ = tokens.delete(existing.id()).await?;
        Ok(())
    } else {
        Ok(())
    }
}

#[openapi(tag = "Users")]
#[get("/self")]
async fn get_user_self(user: User) -> crate::ApiResult<GenericUser> {
    Ok(Json(user.into()))
}

export_routes![login, logout, get_user_self];
