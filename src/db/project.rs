use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::{prelude::FromRow, query_as, types::Json, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, offset_limit};

use super::{
  handle_pg_error, maybe_order_by, sqlx_macro::maybe_bind, CreateResult, Error, UpdateResult,
};

pub type Networks = HashMap<String, String>;

#[derive(FromRow, Serialize)]
pub struct Project {
  pub id: Uuid,
  pub org_id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub website: Option<String>,
  pub networks: Json<Networks>,
  pub feature_from: Option<NaiveDate>,
  pub feature_until: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub description: Option<String>,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ProjectFilter {
  pub org_id: Uuid,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

// list projects
pub async fn list(db: &PgPool, p: super::ListParams<ProjectFilter>) -> Result<Vec<Project>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      id,
      org_id,
      name,
      logo,
      images,
      website,
      networks,
      feature_from,
      feature_until,
      created_at,
      updated_at,
      description
    FROM project
    WHERE"#,
  );

  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = p.filter.org_id);
  maybe_bind!(sep, "created_at" = p.filter.created_after);
  maybe_bind!(sep, "created_at" = p.filter.created_before);

  maybe_order_by(
    &mut query,
    &p.order,
    vec!["id", "name", "created_at", "updated_at"],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a project
pub async fn get(db: &PgPool, org_id: Uuid, project_id: Uuid) -> Result<Project, Error> {
  query_as!(
    Project,
    r#"SELECT
      id,
      org_id,
      name,
      logo,
      images,
      website,
      networks AS "networks: Json<Networks>",
      feature_from,
      feature_until,
      created_at,
      updated_at,
      description
    FROM project
    WHERE org_id = $1 AND id = $2"#,
    org_id,
    project_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize)]
pub struct CreateParam {
  pub org_id: Uuid,
  pub id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: String,
  pub website: String,
  pub networks: Networks,
}

// create a project
pub async fn create(db: &PgPool, p: CreateParam) -> Result<CreateResult, Error> {
  query_as(
    "INSERT INTO project (
      org_id,
      id,
      name,
      logo,
      images,
      description,
      website,
      networks
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    RETURNING created_at",
  )
  .bind(p.org_id)
  .bind(p.id)
  .bind(p.name)
  .bind(p.logo)
  .bind(p.images)
  .bind(p.description)
  .bind(p.website)
  .bind(Json(p.networks))
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[serde_as]
#[derive(Deserialize, IsEmpty)]
pub struct UpdateParams {
  pub name: Option<String>,
  pub logo: Option<String>,
  pub images: Option<Vec<String>>,
  pub description: Option<String>,
  pub website: Option<String>,
  pub networks: Option<Networks>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub feature_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub feature_until: Option<NaiveDate>,
}

// update a project
pub async fn update(
  db: &PgPool,
  org_id: Uuid,
  project_id: Uuid,
  p: UpdateParams,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut query = QueryBuilder::<Postgres>::new("UPDATE project SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "name" = p.name);
  maybe_bind!(sep, "logo" = p.logo);
  maybe_bind!(sep, "images" = p.images);
  maybe_bind!(sep, "description" = p.description);
  maybe_bind!(sep, "website" = p.website);
  maybe_bind!(sep, "networks" = p.networks, Json);
  maybe_bind!(sep, "feature_from" = p.feature_from);
  maybe_bind!(sep, "feature_until" = p.feature_until);
  query.push(" WHERE org_id = ").push_bind(org_id);
  query.push(" AND id = ").push_bind(project_id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

#[serde_as]
#[derive(Deserialize)]
pub struct ReplaceParams {
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: String,
  pub website: String,
  pub networks: Networks,
  #[serde_as(as = "NoneAsEmptyString")]
  pub feature_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub feature_until: Option<NaiveDate>,
}

// replace a project
pub async fn replace(
  db: &PgPool,
  org_id: Uuid,
  project_id: Uuid,
  p: ReplaceParams,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE project SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "name" = p.name);
  must_bind!(sep, "logo" = p.logo);
  must_bind!(sep, "images" = p.images);
  must_bind!(sep, "description" = p.description);
  must_bind!(sep, "website" = p.website);
  must_bind!(sep, "networks" = Json(p.networks));
  maybe_bind!(sep, "feature_from" = p.feature_from);
  maybe_bind!(sep, "feature_until" = p.feature_until);
  query.push(" WHERE org_id = ").push_bind(org_id);
  query.push(" AND id = ").push_bind(project_id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a project
pub async fn delete(db: &PgPool, org_id: Uuid, project_id: Uuid) -> Result<(), Error> {
  match sqlx::query!(
    "DELETE FROM project WHERE org_id = $1 AND id = $2",
    org_id,
    project_id
  )
  .execute(db)
  .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}
