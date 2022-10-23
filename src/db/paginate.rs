use std::fmt::Display;

use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sql_types::BigInt;

const DEFAULT_LIMIT: i64 = 10;

pub trait Paginate: Sized {
    fn paginate(self, offset: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, offset: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            offset,
            limit: DEFAULT_LIMIT,
            column: "id",
            direction: Direction::DESC,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    offset: i64,
    limit: i64,
    column: &'static str,
    direction: Direction,
}

impl<T> Paginated<T> {
    pub fn limit(self, limit: i64) -> Self {
        Paginated { limit, ..self }
    }

    pub fn column(self, column: &'static str) -> Self {
        Paginated { column, ..self }
    }

    pub fn direction(self, direction: Direction) -> Self {
        Paginated { direction, ..self }
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = T::SqlType;
}

impl<Conn, T> RunQueryDsl<Conn> for Paginated<T> where Conn: Connection {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t");
        out.push_sql(format!(" ORDER BY {} {} LIMIT ", self.column, self.direction).as_str());
        out.push_bind_param::<BigInt, _>(&self.limit)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    ASC,
    DESC,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.sign_plus() {
            false => match self {
                Self::ASC => write!(f, "ASC"),
                Self::DESC => write!(f, "DESC"),
            },
            true => match self {
                Self::ASC => write!(f, ">"),
                Self::DESC => write!(f, "<"),
            },
        }
    }
}

impl Direction {}
