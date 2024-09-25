use std::collections::HashMap;

use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, types::Json, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{
  must_bind, maybe_bind, offset_limit,
};

use super::{
  handle_pg_error, maybe_order_by, CreateParam, CreateResult,
  CreatedFilter, Error, ListParams, UpdateResult,
};

#[derive(FromRow, Serialize, Debug)]
pub struct Org {
  pub id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub admins: Json<HashMap<String, i64>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

// list orgs
pub async fn list(
  db: &PgPool,
  user_id: &str,
  p: ListParams<CreatedFilter>,
) -> Result<Vec<Org>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    "SELECT id, name, logo, admins, created_at, updated_at FROM org WHERE",
  );

  let mut sep = query.separated(" AND ");
  // must_bind!(sep, "admins" ? user_id);
  must_bind!(sep, "admins" ? user_id);
  maybe_bind!(sep, "created_at" <= p.filter.created_before);
  maybe_bind!(sep, "created_at" >= p.filter.created_after);

  maybe_order_by(&mut query, &p.order, vec!["created_at"])?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a org
pub async fn get(db: &PgPool, id: Uuid) -> Result<Org, Error> {
  let res = query!(
    r#"SELECT
      name,
      logo,
      admins AS "admins: Json<HashMap<String, i64>>",
      created_at,
      updated_at
    FROM org
    WHERE id = $1"#,
    id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(Org {
    id,
    name: res.name,
    logo: res.logo,
    admins: res.admins,
    created_at: res.created_at,
    updated_at: res.updated_at,
  })
}

#[derive(Deserialize, Default, Debug)]
pub struct CreateForm {
  pub name: String,
  pub logo: Option<String>,
  pub admins: HashMap<String, i64>,
}

// create a org
pub async fn create<'a>(
  db: &PgPool,
  p: CreateParam<'a, CreateForm>,
) -> Result<CreateResult, Error> {
  query_as!(
    CreateResult,
    "INSERT INTO org (id, name, logo, admins) VALUES ($1, $2, $3, $4) RETURNING created_at",
    p.id,
    p.form.name,
    p.form.logo,
    Json(&p.form.admins) as _,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize, IsEmpty, Default)]
pub struct UpdateParam {
  pub name: Option<String>,
  pub logo: Option<String>,
  pub admins: Option<HashMap<String, i64>>,
}

// update a org
pub async fn update(db: &PgPool, org_id: Uuid, p: UpdateParam) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }

  let mut query = QueryBuilder::<Postgres>::new("UPDATE org SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "name" = p.name);
  maybe_bind!(sep, "logo" = p.logo);
  maybe_bind!(sep, "admins" = p.admins, Json);
  query.push(" WHERE id = ").push_bind(org_id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

#[derive(Deserialize)]
pub struct ReplaceParams {
  pub name: String,
  pub logo: Option<Vec<String>>,
}

// replace a org
pub async fn replace(db: &PgPool, id: Uuid, p: ReplaceParams) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE org SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "p.name" = p.name);
  must_bind!(sep, "p.logo" = p.logo);
  query.push(" WHERE id = ").push_bind(id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a org
pub async fn delete(db: &PgPool, org_id: Uuid) -> Result<(), Error> {
  match query!("DELETE FROM org WHERE id = $1", org_id)
    .execute(db)
    .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}
