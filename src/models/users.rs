use crate::{
    db,
    schema::{posts, tokens, users},
    ApiError, Post,
};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Insertable, Validate)]
#[diesel(table_name = users)]
pub struct Registration {
    #[validate(custom = "User::valid_username")]
    pub username: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = tokens)]
pub struct NewToken {
    pub user: Uuid,
    pub expiration: NaiveDateTime,
}

impl NewToken {
    pub fn new(user: Uuid) -> Result<Self, ApiError> {
        Ok(NewToken {
            user,
            expiration: Utc::now()
                .checked_add_signed(Duration::days(14))
                .ok_or_else(|| ApiError::new(
                    500, "Couldn't compute expiration date of token".into(),
                ))?
                .naive_utc(),
        })
    }
}

/// A token is a secret used to authenticate a user and is created when the user
/// logs in.
#[derive(Debug, Queryable, Serialize)]
pub struct Token {
    /// The id of the token.
    pub id: Uuid,
    /// The id of the user the token belongs to.
    #[serde(skip_serializing)]
    pub user: Uuid,
    /// The expiration date of the token.
    pub expiration: NaiveDateTime,
    /// The date the token was created.
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    /// The date the token was last updated.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<NaiveDateTime>,
}

impl Token {
    /// Finds a `Token` by its `id`.
    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        let token = tokens::table
            .filter(tokens::id.eq(id))
            .first::<Token>(&mut db::connection()?)?;

        if token.expiration < Utc::now().naive_utc() {
            diesel::delete(tokens::table.filter(tokens::id.eq(id)))
                .execute(&mut db::connection()?)?;
            Err(ApiError::new(404, "Token has expired.".into()))
        } else {
            Ok(token)
        }
    }

    /// Deletes a `Token`.
    pub fn delete(&self) -> Result<Self, ApiError> {
        Ok(diesel::delete(tokens::table)
            .filter(tokens::id.eq(self.id))
            .get_result(&mut db::connection()?)?)
    }
}

impl TryFrom<NewToken> for Token {
    type Error = ApiError;

    fn try_from(token: NewToken) -> Result<Self, Self::Error> {
        let token = diesel::insert_into(tokens::table)
            .values(token)
            .get_result(&mut db::connection()?)?;
        Ok(token)
    }
}

/// A session has a token used for authentication.
#[derive(Debug, Deserialize)]
pub struct Session {
    pub token: Uuid,
}

#[derive(Debug, Queryable, Serialize)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub about: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing)]
    pub updated_at: Option<NaiveDateTime>,
}

impl User {
    /// Finds a `User` by its `id`.
    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        Ok(users::table
            .filter(users::id.eq(id))
            .first(&mut db::connection()?)?)
    }

    /// Finds a `User` by its `name`.
    pub fn by_name(username: String) -> Result<Self, ApiError> {
        Ok(users::table
            .filter(users::username.eq(username))
            .first(&mut db::connection()?)?)
    }

    /// Finds a `User`'s posts.
    pub fn posts(&self) -> Result<Vec<Post>, ApiError> {
        Ok(posts::table
            .filter(posts::author.eq(self.username.to_owned()))
            .load(&mut db::connection()?)?)
    }

    /// Generates and returns a new `Token`.
    pub fn get_token(&self) -> Result<Token, ApiError> {
        Token::try_from(NewToken::new(self.id)?)
    }

    /// Updates the users preferences.
    pub fn update(&self, update: UserUpdate) -> Result<Self, ApiError> {
        Ok(diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(update)
            .get_result(&mut db::connection()?)?)
    }

    /// Returns the `User` the `Token` belongs to if the `Token` is valid and an
    /// `ApiError` if it is unknown (or expired, in which case it is also
    /// deleted).
    pub fn from_token(token: Uuid) -> Result<Self, ApiError> {
        let token = Token::find(token).map_err(|e| {
            if e.status_code == 404 {
                ApiError::new(401, "Unknown token.".into())
            } else {
                e
            }
        })?;

        Self::find(token.user)
    }

    /// Checks if the given `&str` constitutes a valid username.
    ///
    /// A valid username
    /// - is at least 3 characters long,
    /// - is at most 20 characters long,
    /// - contains only letters, digits, underscores, and periods, and
    /// - contains no underscore or period following an underscore or a period.
    ///
    /// ```
    /// use ephemeris::User;
    ///
    /// assert!(User::valid_username("_a.").is_ok());
    ///
    /// // "_." is not allowed!
    /// assert!(User::valid_username("_.a").is_err());
    /// ```
    pub fn valid_username(username: &str) -> Result<(), ValidationError> {
        match username {
            s if s.len() < 3 => Err(ValidationError::new(
                "Username must be at  least 3 characters long.",
            )),
            s if s.len() > 20 => Err(ValidationError::new(
                "Username can be at most 20 characters long.",
            )),
            s if !s.to_ascii_lowercase().chars().all(|c|
                "abcdefghijklmnopqrstuvwxyz0123456789._".contains(c))
            => Err(ValidationError::new(
                "Username can contain only letters, digits, underscores, and \
                periods.",
            )),
            s if s.contains("..") || s.contains("._") || s.contains("_.") ||
                s.contains("__")
            => Err(ValidationError::new("Username \
                can't contain an underscore or a period following an \
                underscore or a period.")),
            _ => Ok(())
        }
    }
}

impl TryFrom<Login> for User {
    type Error = ApiError;

    fn try_from(login: Login) -> Result<Self, Self::Error> {
        let user = User::by_name(login.username)
            .map_err(|_| ApiError::new(404, "Unknown user.".into()))?;

        if argon2::verify_encoded(&user.password, login.password.as_bytes())
            .map_err(|e| ApiError::new(500, format!("Couldn't verify hash: {}", e)))?
        {
            info!("{:?} logged in", user.username);
            Ok(user)
        } else {
            Err(ApiError::new(401, "Invalid password.".into()))
        }
    }
}

impl TryFrom<Registration> for User {
    type Error = ApiError;

    fn try_from(user: Registration) -> Result<Self, Self::Error> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = argon2::Config::default();
        let hash = argon2::hash_encoded(user.password.as_bytes(), &salt, &config)
            .map_err(|e| ApiError::new(500, format!("Couldn't hash password: {}", e)))?;
        let user = Registration {
            password: hash,
            username: user.username.to_ascii_lowercase(),
        };

        if users::table
                .filter(users::username.eq(user.username.to_owned()))
                .first::<User>(&mut db::connection()?)
                .is_ok() {
            return Err(ApiError::new(409, "Username already in use.".into()));
        }

        let user = diesel::insert_into(users::table)
            .values(user)
            .get_result::<Self>(&mut db::connection()?)?;

        info!("Registered {:?}", user.username);

        Ok(user)
    }
}

#[derive(Debug, AsChangeset, Deserialize, Validate)]
#[diesel(table_name = users)]
pub struct UserUpdate {
    #[validate(length(max = 160))]
    pub about: Option<String>,
}
