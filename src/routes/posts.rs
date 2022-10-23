use crate::{ApiError, NewPost, Post, PostFilters, User, Session};
use actix_web::{
    get, post, delete,
    web::{self, Json, Path},
    HttpResponse,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[get("/posts")]
async fn find_all(filters: web::Query<PostFilters>) -> Result<HttpResponse, ApiError> {
    filters.validate()?;
    let posts = Post::find_all(filters.into_inner())?;
    Ok(HttpResponse::Ok().json(posts))
}

#[get("/post/{id}")]
async fn find(id: Path<i32>) -> Result<HttpResponse, ApiError> {
    let post = Post::find(id.into_inner())?;
    Ok(HttpResponse::Ok().json(post))
}

#[delete("/post/{id}")]
async fn delete(
    id: Path<i32>, session: Json<Session>,
) -> Result<HttpResponse, ApiError> {
    let Session { token } = session.into_inner();
    let user = User::from_token(token)?;
    let post = Post::find(id.into_inner())?;

    if user.username != post.author {
        Err(ApiError::new(403, "You can't delete this post.".into()))
    } else {
        post.delete()?;
        Ok(HttpResponse::NoContent().finish())
    }
}

#[derive(Deserialize)]
struct CreateMessage {
    pub post: NewPost,
    pub token: Uuid,
}

#[post("/post")]
async fn create(data: Json<CreateMessage>) -> Result<HttpResponse, ApiError> {
    let CreateMessage { post, token } = data.into_inner();
    post.validate()?;
    let author = User::from_token(token)?;
    let post = Post::try_from((post, &author))?;
    Ok(HttpResponse::Created().json(post))
}

#[derive(Deserialize)]
struct EditMessage {
    pub id: i32,
    pub post: NewPost,
    pub token: Uuid,
}

#[post("/edit")]
async fn edit(data: Json<EditMessage>) -> Result<HttpResponse, ApiError> {
    let EditMessage { id, post, token } = data.into_inner();
    post.validate()?;
    let old_post = Post::find(id)?;
    let author = User::from_token(token)?;
    if author.username != old_post.author {
        return Err(ApiError::new(403, "You can't edit this post.".into()));
    }
    Ok(HttpResponse::Ok().json(old_post.edit(post)?))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find);
    cfg.service(delete);
    cfg.service(create);
    cfg.service(edit);
}
