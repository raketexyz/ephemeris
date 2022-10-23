use crate::db::Paginate;
use crate::schema::posts;
use crate::{db, ApiError, User};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate, AsChangeset)]
#[diesel(table_name = posts)]
pub struct NewPost {
    #[validate(length(min = 1, max = 100))]
    pub title: String,
    #[validate(length(max = 140))]
    pub subtitle: String,
    pub body: String,
}

#[derive(Queryable, Serialize)]
pub struct Post {
    pub id: i32,
    pub author: String,
    pub title: String,
    pub subtitle: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct InsertablePost {
    pub author: String,
    pub title: String,
    pub subtitle: String,
    pub body: String,
}

/// Filters to be applied to a post search.
#[derive(Debug, Deserialize, Validate)]
pub struct PostFilters {
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    #[validate(range(min = 0))]
    pub limit: Option<i64>,
    pub author: Option<String>,
}

impl Post {
    /// Returns all posts matching the filters.
    pub fn find_all(filters: PostFilters) -> Result<Vec<Self>, ApiError> {
        let mut query = match filters.author {
            Some(s) => posts::table.filter(posts::author.eq(s)).into_boxed(),
            None => posts::table.into_boxed(),
        }
        .paginate(filters.offset.unwrap_or(0));

        if let Some(limit) = filters.limit {
            query = query.limit(limit)
        }

        let posts = query.get_results(&mut db::connection()?)?;

        Ok(posts)
    }

    /// Finds a post by its id.
    pub fn find(id: i32) -> Result<Self, ApiError> {
        Ok(posts::table
            .filter(posts::id.eq(id))
            .first(&mut db::connection()?)?)
    }

    /// Updates the post with the supplied new post.
    pub fn edit(&self, update: NewPost) -> Result<Self, ApiError> {
        Ok(diesel::update(posts::table.filter(posts::id.eq(self.id)))
            .set(update)
            .get_result(&mut db::connection()?)?)
    }

    /// Deletes the post.
    pub fn delete(&self) -> Result<Self, ApiError> {
        Ok(diesel::delete(posts::table.filter(posts::id.eq(self.id)))
           .get_result(&mut db::connection()?)?)
    }
}

impl TryFrom<(NewPost, &User)> for Post {
    type Error = ApiError;

    fn try_from((mut post, author): (NewPost, &User)) -> Result<Self, Self::Error> {
        post.body = post.body.trim().to_string();

        let post = diesel::insert_into(posts::table)
            .values(&InsertablePost {
                author: author.username.to_owned(),
                title: post.title,
                subtitle: post.subtitle,
                body: post.body,
            })
            .get_result::<Post>(&mut db::connection()?)?;

        info!("{:?} posted {:?} (#{})", author.username, post.title, post.id);

        Ok(post)
    }
}
