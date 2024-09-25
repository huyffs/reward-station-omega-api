use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{prelude::FromRow, query_as, types::Json, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{maybe_bind, offset_limit};

use super::{
  campaign::{CampaignFilter, Task},
  handle_pg_error, maybe_order_by, Error,
};

#[derive(FromRow, Serialize)]
pub struct PubCampaign {
  pub org_id: Option<Uuid>,
  pub project_id: Uuid,
  pub id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: Option<String>,
  pub chain_id: i64,
  pub contract_address: String,
  pub condition_info: Option<String>,
  pub reward_info: Option<String>,
  pub tasks: Json<Vec<Task>>,
  pub start_at: Option<NaiveDate>,
  pub end_at: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

// list campaigns
pub async fn list(
  db: &PgPool,
  p: super::ListParams<CampaignFilter>,
) -> Result<Vec<PubCampaign>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      org_id,
      project_id,
      id,
      name,
      logo,
      images,
      description,
      coupon_code,
      budget,
      chain_id,
      contract_address,
      condition_info,
      reward_info,
      tasks,
      start_at,
      end_at,
      created_at,
      updated_at
    FROM campaign"#,
  );

  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  sep.push("start_at <= NOW()");
  sep.push("(end_at IS NULL OR end_at >= NOW())");
  maybe_bind!(sep, "org_id" = p.filter.org_id);
  maybe_bind!(sep, "project_id" = p.filter.project_id);
  maybe_bind!(sep, "created_at" <= p.filter.created_before);
  maybe_bind!(sep, "created_at" >= p.filter.created_after);
  maybe_bind!(
    sep,
    "contract_address"
    IN
    p.filter.contracts,
    |c: Vec<String>| c
      .into_iter()
      .map(|s| s.to_lowercase())
      .collect::<Vec<String>>()
  );
  maybe_order_by(
    &mut query,
    &p.order,
    vec![
      "id",
      "name",
      "reward_amount",
      "start_at",
      "end_at",
      "created_at",
      "updated_at",
    ],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a campaign
pub async fn get(db: &PgPool, campaign_id: Uuid) -> Result<PubCampaign, Error> {
  query_as!(
    PubCampaign,
    r#"SELECT
      org_id,
      project_id,
      id,
      name,
      logo,
      images,
      description,
      chain_id,
      contract_address,
      condition_info,
      reward_info,
      tasks AS "tasks: Json<Vec<Task>>",
      start_at,
      end_at,
      created_at,
      updated_at
    FROM campaign
    WHERE id = $1
      AND start_at <= NOW()
      AND (end_at IS NULL OR end_at >= NOW())
    "#,
    campaign_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(FromRow, Serialize)]
pub struct GetOrgIdResult {
  pub org_id: Uuid,
}

// get a campaign's org id
pub async fn get_org_id(db: &PgPool, campaign_id: Uuid) -> Result<GetOrgIdResult, Error> {
  query_as!(
    GetOrgIdResult,
    r#"SELECT org_id FROM campaign WHERE id = $1"#,
    campaign_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(FromRow, Serialize)]
pub struct GetRelationIdResult {
  pub org_id: Uuid,
  pub project_id: Uuid,
}

// get a campaign's org id
pub async fn get_relation_ids(
  db: &PgPool,
  campaign_id: Uuid,
) -> Result<GetRelationIdResult, Error> {
  query_as!(
    GetRelationIdResult,
    r#"SELECT org_id, project_id FROM campaign WHERE id = $1"#,
    campaign_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}
