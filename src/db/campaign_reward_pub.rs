use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::{
  maybe_order_by,
  sqlx_macro::{must_bind, maybe_bind, offset_limit},
};

use super::{handle_pg_error, Error};

#[derive(FromRow, Serialize)]
pub struct PubCampaignReward {
  pub id: Uuid,
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub tandc: Option<String>,
  pub images: Vec<String>,
  pub active_from: Option<NaiveDate>,
  pub active_until: Option<NaiveDate>,
  pub valid_from: Option<NaiveDate>,
  pub valid_until: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub org_id: Uuid,
  pub active: bool,
  pub point: i64,
  pub max_mint: Option<i64>,
  pub user_mint: Option<i64>,
  pub coupon_minted: i64,
  pub coupon_total: i64,
}

// get rewards
pub async fn get(
  db: &PgPool,
  campaign_id: Uuid,
  reward_id: Uuid,
) -> Result<PubCampaignReward, Error> {
  query_as!(
    PubCampaignReward,
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
      pr.campaign_id,
      pr.point,
      pr.active,
      pr.max_mint,
      pr.user_mint,
      (SELECT COUNT(user_id) as "coupon_minted!" FROM coupon WHERE reward_id = r.id),
      (SELECT COUNT(*) as "coupon_total!" FROM coupon WHERE reward_id = r.id)
    FROM reward r
    LEFT JOIN campaign__reward pr
      ON r.id = pr.reward_id
    WHERE pr.campaign_id = $1
      AND r.id = $2
    "#,
    campaign_id,
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
  campaign_id: Uuid,
  p: super::ListParams<RewardFilter>,
) -> Result<Vec<PubCampaignReward>, Error> {
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
      pr.campaign_id,
      pr.point,
      pr.active,
      pr.max_mint,
      pr.user_mint,
      (SELECT COUNT(user_id) as "coupon_minted" FROM coupon WHERE reward_id = r.id),
      (SELECT COUNT(*) as "coupon_total" FROM coupon WHERE reward_id = r.id)
    FROM reward r
    LEFT JOIN campaign__reward pr
    ON r.id = pr.reward_id"#,
  );

  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  sep.push(" (r.active_until IS NULL OR r.active_until >= NOW())");
  sep.push(" r.active_from <= NOW()");
  must_bind!(sep, "pr.campaign_id" = campaign_id);
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
