use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;
use serde_with::{serde_as, NoneAsEmptyString};

use crate::db::{
  maybe_order_by,
  sqlx_macro::{must_bind, maybe_bind, offset_limit},
};

use super::{handle_pg_error, Error};

#[serde_as]
#[derive(FromRow, Serialize)]
pub struct PubProjectReward {
  pub id: Uuid,
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub tandc: Option<String>,
  pub images: Vec<String>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_until: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_until: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub project_id: Uuid,
  pub org_id: Uuid,
  pub active: bool,
  pub point: i64,
}

// get rewards
pub async fn get(
  db: &PgPool,
  project_id: Uuid,
  reward_id: Uuid,
) -> Result<PubProjectReward, Error> {
  query_as!(
    PubProjectReward,
    r#"SELECT
      r.id,
      r.issuer_id,
      r.category,
      r.country_id,
      r.name,
      r.tandc,
      r.images,
      r.active_from,
      r.active_until,
      r.valid_from,
      r.valid_until,
      r.created_at,
      r.updated_at,
      pr.org_id,
      pr.project_id,
      pr.point,
      pr.active
    FROM reward r
    LEFT JOIN project__reward pr
      ON r.id = pr.reward_id
    WHERE pr.project_id = $1
      AND r.id = $2
    "#,
    project_id,
    reward_id,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize, Clone, Debug, IsEmpty)]
pub struct RewardFilter {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
  pub active: Option<bool>,
}

// list rewards
pub async fn list(
  db: &PgPool,
  project_id: Uuid,
  p: super::ListParams<RewardFilter>,
) -> Result<Vec<PubProjectReward>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      r.id,
      r.issuer_id,
      r.category,
      r.country_id,
      r.name,
      r.tandc,
      r.images,
      r.active_from,
      r.active_until,
      r.valid_from,
      r.valid_until,
      r.created_at,
      r.updated_at,
      pr.org_id,
      pr.project_id,
      pr.point,
      pr.active
    FROM reward r
    LEFT JOIN project__reward pr
    ON r.id = pr.reward_id"#,
  );

  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  sep.push(" (r.active_until IS NULL OR r.active_until >= NOW())");
  sep.push(" r.active_from <= NOW()");
  must_bind!(sep, "pr.project_id" = project_id);
  maybe_bind!(sep, "r.issuer_id" = p.filter.issuer_id);
  maybe_bind!(sep, "r.category" = p.filter.category);
  maybe_bind!(sep, "r.country_id" = p.filter.country_id);
  maybe_bind!(sep, "r.created_at" <= p.filter.created_before);
  maybe_bind!(sep, "r.created_at" >= p.filter.created_after);
  maybe_bind!(sep, "r.active" >= p.filter.active);

  maybe_order_by(
    &mut query,
    &p.order,
    vec![
      "r.active_from",
      "r.active_until",
      "r.valid_from",
      "r.valid_until",
      "r.created_at",
      "r.updated_at",
      "pr.point",
    ],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}
