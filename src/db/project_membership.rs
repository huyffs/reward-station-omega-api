use serde::{Deserialize, Serialize};
use serde_with::{formats::CommaSeparator, serde_as, skip_serializing_none, StringWithSeparator};
use sqlx::{prelude::FromRow, query, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::{handle_pg_error, sqlx_macro::must_bind};

use super::ListParams;

#[skip_serializing_none]
#[derive(Serialize, FromRow)]
pub struct ProjectMembership {
  pub project_id: Uuid,
  pub user_id: String,
  pub subscribed: Option<bool>,
  pub balance: i64,
  pub point: i64,
}

#[serde_as]
#[derive(Deserialize, Clone, Debug)]
pub struct ProjectMembershipFilter {
  #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, Uuid>>")]
  pub project_ids: Option<Vec<Uuid>>,
  pub subscribe: Option<bool>,
}

pub async fn list(
  db: &PgPool,
  user_id: &str,
  p: ListParams<ProjectMembershipFilter>,
) -> Result<Vec<ProjectMembership>, super::Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      project_id,
      user_id,
      subscribed,
      balance,
      point
    FROM project__user p
    RIGHT JOIN (
      SELECT
        project_id,
        user_id,
        SUM(balance)::BIGINT AS balance,
        SUM(value)::BIGINT AS point
      FROM voucher
      GROUP BY (project_id, user_id)
    ) AS v
    USING (project_id, user_id)
    WHERE"#,
  );
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "user_id" = user_id);
  if let Some(value) = p.filter.project_ids {
    sep
      .push("project_id = ANY(")
      .push_bind_unseparated(value)
      .push_unseparated(")");
  }

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

#[derive(Serialize)]
pub struct JoinResult {
  subscribed: bool,
}

pub async fn join(
  db: &PgPool,
  project_id: Uuid,
  user_id: &str,
) -> Result<JoinResult, super::Error> {
  _ = query!(
    r#"INSERT INTO project__user (
      project_id,
      user_id,
      subscribed
    )
    VALUES ($1, $2, true)
    ON CONFLICT(project_id, user_id) 
    DO UPDATE SET
      subscribed = true"#,
    project_id,
    user_id,
  )
  .execute(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(JoinResult { subscribed: true })
}
