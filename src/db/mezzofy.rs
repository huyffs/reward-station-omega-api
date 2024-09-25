use super::{handle_pg_error, sqlx_macro::maybe_bind, Error, UpdateResult};

use crate::rec_http::Headers;

use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, types::Json, PgPool, Postgres, QueryBuilder};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MezzofyCallLog {
  pub id: String,
  pub idemp_key: Option<String>,
  pub method: String,
  pub url: String,
  pub req_headers: Json<Headers>,
  pub req_body: Option<String>,
  pub res_headers: Option<Json<Headers>>,
  pub res_body: Option<String>,
  pub status: Option<i32>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct CreateParam<'a> {
  pub idemp_key: Option<&'a str>,
  pub method: &'a str,
  pub url: &'a str,
  pub req_headers: &'a Headers,
  pub req_body: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct CreateResult {
  pub id: i64,
  pub created_at: DateTime<Utc>,
}

pub async fn create<'a>(db: &PgPool, p: CreateParam<'a>) -> Result<CreateResult, Error> {
  query_as!(
    CreateResult,
    "INSERT INTO mezzofy_call_log (
      idemp_key,
      method,
      url,
      req_headers,
      req_body
    )
    VALUES ($1, $2, $3, $4, $5)
    RETURNING id, created_at",
    p.idemp_key,
    p.method,
    p.url,
    Json(&p.req_headers) as _,
    p.req_body,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(IsEmpty)]
pub struct UpdateParams<'a> {
  pub res_headers: Option<&'a Headers>,
  pub res_body: Option<&'a str>,
  pub status: Option<i16>,
}

// update a project
pub async fn update<'a>(db: &PgPool, id: i64, p: UpdateParams<'a>) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut query = QueryBuilder::<Postgres>::new("UPDATE mezzofy_call_log SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "res_headers" = p.res_headers, Json);
  maybe_bind!(sep, "res_body" = p.res_body);
  maybe_bind!(sep, "status" = p.status);
  query.push(" WHERE id = ").push_bind(id);
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}
