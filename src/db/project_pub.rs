use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::Serialize;
use sqlx::{prelude::FromRow, query_as, types::Json, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::offset_limit;

use super::{
  handle_pg_error, maybe_order_by, project::Networks, sqlx_macro::maybe_bind, CreatedFilter,
  Error,
};

#[derive(FromRow, Serialize, Debug)]
pub struct PubProject {
  pub id: Uuid,
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

// list projects
pub async fn list(
  db: &PgPool,
  p: super::ListParams<CreatedFilter>,
) -> Result<Vec<PubProject>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      id,
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
    FROM project"#,
  );

  if !p.filter.is_empty() {
    query.push(" WHERE ");
    let mut sep = query.separated(" AND ");
    maybe_bind!(sep, "created_at" <= p.filter.created_before);
    maybe_bind!(sep, "created_at" >= p.filter.created_after);
  }

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
pub async fn get(db: &PgPool, project_id: Uuid) -> Result<PubProject, Error> {
  query_as!(
    PubProject,
    r#"SELECT
      id,
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
    WHERE id = $1"#,
    project_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}
