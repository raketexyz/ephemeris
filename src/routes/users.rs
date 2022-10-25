use crate::{ApiError, Login, Registration, Token, User, Session, UserUpdate};
use serde::Deserialize;
use uuid::Uuid;
use actix_web::{
    get, post,
    web::{self, Json, Path},
    HttpResponse,
};
use serde_json::json;
use validator::Validate;

#[get("/user/{username}")]
async fn find(username: Path<String>) -> Result<HttpResponse, ApiError> {
    let user = User::by_name(username.into_inner())?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/user")]
async fn register(form: Json<Registration>) -> Result<HttpResponse, ApiError> {
    let form = form.into_inner();
    form.validate()?;
    let user = User::try_from(form)?;
    Ok(HttpResponse::Created().json(user))
}

#[post("/login")]
async fn login(form: Json<Login>) -> Result<HttpResponse, ApiError> {
    let form = form.into_inner();
    let user = User::try_from(form)?;
    let token = user.get_token()?;
    Ok(HttpResponse::Ok().json(token))
}

#[derive(Deserialize)]
struct UpdateMessage {
    update: UserUpdate,
    token: Uuid,
}

#[post("/preferences")]
async fn update(data: Json<UpdateMessage>) -> Result<HttpResponse, ApiError> {
    let UpdateMessage { update, token } = data.into_inner();
    update.validate()?;
    let user = User::from_token(token)?;
    Ok(HttpResponse::Ok().json(user.update(update)?))
}

#[get("/logout")]
async fn logout(session: web::Query<Session>) -> Result<HttpResponse, ApiError> {
    let token = Token::find(session.into_inner().token)?;
    token.delete()?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/session")]
async fn get_session(session: web::Query<Session>) -> Result<HttpResponse, ApiError> {
    let Session { token } = session.into_inner();
    let user = User::from_token(token)?;

    Ok(HttpResponse::Ok().json(json!({
        "expires": Token::find(token)?.expiration,
        "user": user
    })))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find);
    cfg.service(register);
    cfg.service(login);
    cfg.service(update);
    cfg.service(logout);
    cfg.service(get_session);
}
