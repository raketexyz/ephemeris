use crate::{schema::comments, ApiError, db::{Paginate, self}, User};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct NewComment {
    pub post: i32,
    #[validate(length(min = 1, max = 140))]
    pub message: String,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Comment {
    pub id: i32,
    pub author: String,
    pub post: i32,
    pub message: String,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Validate, AsChangeset)]
#[diesel(table_name = comments)]
pub struct UpdateComment {
    pub id: i32,
    #[validate(length(min = 1, max = 140))]
    pub message: String,
}

impl Comment {
    /// Returns all comments matching the filters.
    pub fn find_all(filters: CommentFilters) -> Result<Vec<Self>, ApiError> {
        let mut query = comments::table.filter(comments::post.eq(filters.post))
            .paginate(filters.offset.unwrap_or(0));

        if let Some(limit) = filters.limit {
            query = query.limit(limit)
        }

        let comments = query.get_results(&mut db::connection()?)?;

        Ok(comments)
    }

    /// Finds a comments by its id.
    pub fn find(id: i32) -> Result<Self, ApiError> {
        Ok(comments::table
           .filter(comments::id.eq(id))
           .first(&mut db::connection()?)?)
    }

    /// Updates the comment with the supplied new comment.
    pub fn edit(&self, update: UpdateComment) -> Result<Self, ApiError> {
        Ok(diesel::update(comments::table.filter(comments::id.eq(self.id)))
           .set(update)
           .get_result(&mut db::connection()?)?)
    }

    /// Deletes the comment.
    pub fn delete(&self) -> Result<Self, ApiError> {
        Ok(diesel::delete(comments::table.filter(comments::id.eq(self.id)))
           .get_result(&mut db::connection()?)?)
    }
}

impl TryFrom<(NewComment, &User)> for Comment {
    type Error = ApiError;

    fn try_from(
        (mut comment, author): (NewComment, &User)
    ) -> Result<Self, Self::Error> {
        comment.message = comment.message.trim().to_string();
        if comment.message.is_empty() {
            return Err(ApiError::new(400, "Message can't be empty.".into()))
        }

        let comment = diesel::insert_into(comments::table)
            .values(&InsertableComment {
                author: author.username.to_owned(),
                post: comment.post,
                message: comment.message,
            })
            .get_result::<Comment>(&mut db::connection()?)?;

        info!("{:?} posted comment {} on post #{}", author.username, comment.id,
              comment.post);
        Ok(comment)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = comments)]
pub struct InsertableComment {
    pub author: String,
    pub post: i32,
    pub message: String,
}

/// Filters to be applied to a comment search.
#[derive(Debug, Deserialize, Validate)]
pub struct CommentFilters {
    post: i32,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
    #[validate(range(min = 0))]
    pub limit: Option<i64>,
}
