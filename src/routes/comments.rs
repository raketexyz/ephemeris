use actix_web::{get, post, delete, put, web::{self, Path, Json}, HttpResponse};
use uuid::Uuid;
use serde::Deserialize;
use validator::Validate;
use crate::{CommentFilters, ApiError, Comment, Session, User, NewComment, UpdateComment};

#[get("/comments")]
async fn find_all(
    filters: web::Query<CommentFilters>,
) -> Result<HttpResponse, ApiError> {
    let comments = Comment::find_all(filters.into_inner())?;
    Ok(HttpResponse::Ok().json(comments))
}

#[get("/comment/{id}")]
async fn find(id: Path<i32>) -> Result<HttpResponse, ApiError> {
    let comment = Comment::find(id.into_inner())?;
    Ok(HttpResponse::Ok().json(comment))
}

#[delete("/comment/{id}")]
async fn delete(id: Path<i32>, session: Json<Session>) -> Result<HttpResponse, ApiError> {
    let Session { token } = session.into_inner();
    let user = User::from_token(token)?;
    let comment = Comment::find(id.into_inner())?;

    if user.username == comment.author {
        Ok(HttpResponse::Ok().json(comment.delete()?))
    } else {
        Err(ApiError::new(403, "You can't delete this comment.".into()))
    }
}

#[derive(Deserialize)]
struct CreateMessage {
    pub comment: NewComment,
    pub token: Uuid,
}

#[post("/comment")]
async fn create(data: Json<CreateMessage>) -> Result<HttpResponse, ApiError> {
    let CreateMessage { comment, token } = data.into_inner();
    comment.validate()?;
    let author = User::from_token(token)?;
    let comment = Comment::try_from((comment, &author))?;
    Ok(HttpResponse::Created().json(comment))
}

#[derive(Deserialize)]
struct EditMessage {
    pub comment: UpdateComment,
    pub token: Uuid,
}

#[put("/comment")]
async fn edit(data: Json<EditMessage>) -> Result<HttpResponse, ApiError> {
    let EditMessage { comment, token } = data.into_inner();
    comment.validate()?;
    let old_comment = Comment::find(comment.id)?;
    let user = User::from_token(token)?;
    if user.username != old_comment.author {
        return Err(ApiError::new(403, "You can't edit this comment.".into()));
    }
    Ok(HttpResponse::Ok().json(old_comment.edit(comment)?))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(delete);
    cfg.service(create);
    cfg.service(edit);
}
