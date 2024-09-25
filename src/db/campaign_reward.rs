use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::{
  maybe_order_by,
  sqlx_macro::{must_bind, maybe_bind, offset_limit},
  UpdateResult,
};

use super::{handle_pg_error, CreateResult, Error};

#[derive(FromRow, Serialize)]
pub struct CampaignReward {
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
  pub org_id: Option<Uuid>,
  pub project_id: Option<Uuid>,
  pub campaign_id: Option<Uuid>,
  pub approved: Option<bool>,
  pub active: Option<bool>,
  pub point: Option<i64>,
  pub max_mint: Option<i64>,
  pub user_mint: Option<i64>,
  pub link_created_at: Option<DateTime<Utc>>,
  pub link_updated_at: Option<DateTime<Utc>>,
  pub coupon_minted: i64,
  pub coupon_total: i64,
}

// get rewards
pub async fn get(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  reward_id: Uuid,
) -> Result<CampaignReward, Error> {
  query_as!(
    CampaignReward,
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
      pr.approved,
      pr.active,
      pr.max_mint,
      pr.user_mint,
      pr.created_at as link_created_at,
      pr.updated_at as link_updated_at,
      (SELECT COUNT(user_id) as "coupon_minted!" FROM coupon WHERE reward_id = r.id),
      (SELECT COUNT(*) as "coupon_total!" FROM coupon WHERE reward_id = r.id)
    FROM reward r
    LEFT JOIN (
      SELECT *
      FROM campaign__reward
      WHERE org_id = $1
        AND campaign_id = $2
    ) pr
    ON r.id = pr.reward_id
    WHERE r.id = $3"#,
    org_id,
    campaign_id,
    reward_id,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize, IsEmpty, Clone, Debug)]
pub struct CampaignRewardFilter {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
  pub approved: Option<bool>,
  pub active: Option<bool>,
}

// list rewards
pub async fn list(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  p: super::ListParams<CampaignRewardFilter>,
) -> Result<Vec<CampaignReward>, Error> {
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
      pr.approved,
      pr.active,
      pr.max_mint,
      pr.user_mint,
      pr.created_at as link_created_at,
      pr.updated_at as link_updated_at,
      (SELECT COUNT(user_id) as "coupon_minted" FROM coupon WHERE reward_id = r.id),
      (SELECT COUNT(*) as "coupon_total" FROM coupon WHERE reward_id = r.id)
    FROM reward r
    LEFT JOIN (
      SELECT *
      FROM campaign__reward
      WHERE org_id = "#,
  );
  query.push_bind(org_id);
  query.push(" AND campaign_id = ");
  query.push_bind(campaign_id);
  query.push(" ) pr ON r.id = pr.reward_id ");

  if !p.filter.is_empty() {
    query.push(" WHERE ");
    let mut sep = query.separated(" AND ");
    // must_bind!(sep, "pr.org_id" = org_id);
    // must_bind!(sep, "pr.campaign_id" = campaign_id);

    maybe_bind!(sep, "r.issuer_id" = p.filter.issuer_id);
    maybe_bind!(sep, "r.category" = p.filter.category);
    maybe_bind!(sep, "r.country_id" = p.filter.country_id);
    maybe_bind!(sep, "r.created_at" <= p.filter.created_before);
    maybe_bind!(sep, "r.created_at" >= p.filter.created_after);
    maybe_bind!(sep, "r.approved" >= p.filter.approved);
    maybe_bind!(sep, "r.active" >= p.filter.active);
  }

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
      "pr.max_mint",
      "pr.user_mint",
    ],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

#[derive(Deserialize, Default, Debug)]
pub struct CreateParam {
  pub project_id: Uuid,
  pub reward_id: Uuid,
  pub point: i64,
  pub active: bool,
  pub max_mint: Option<i64>,
  pub user_mint: Option<i64>,
}

// create a campaign__reward
pub async fn create(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  p: CreateParam,
) -> Result<CreateResult, Error> {
  if p.point < 0 || p.max_mint.is_some_and(|n| n < 0) || p.user_mint.is_some_and(|n| n < 0) {
    return Err(Error::Validation);
  }
  query_as!(
    CreateResult,
    "INSERT INTO campaign__reward (
      org_id,
      project_id,
      campaign_id,
      reward_id,
      point,
      active,
      max_mint,
      user_mint,
      approved
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true)
    RETURNING created_at",
    org_id,
    p.project_id,
    campaign_id,
    p.reward_id,
    p.point,
    p.active,
    p.max_mint,
    p.user_mint,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize, IsEmpty, Default)]
pub struct UpdateParam {
  pub approved: Option<bool>,
  pub active: Option<bool>,
  pub point: Option<i64>,
  pub max_mint: Option<i64>,
  pub user_mint: Option<i64>,
}

// update a campaign__reward
pub async fn update(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  reward_id: Uuid,
  p: UpdateParam,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  if p.point.is_some_and(|n| n < 0)
    || p.max_mint.is_some_and(|n| n < 0)
    || p.user_mint.is_some_and(|n| n < 0)
  {
    return Err(Error::Validation);
  }

  let mut query = QueryBuilder::<Postgres>::new("UPDATE campaign__reward SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "approved" = p.approved);
  maybe_bind!(sep, "active" = p.active);
  maybe_bind!(sep, "point" = p.point);
  maybe_bind!(sep, "max_mint" = p.max_mint);
  maybe_bind!(sep, "user_mint" = p.user_mint);
  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = org_id);
  must_bind!(sep, "campaign_id" = campaign_id);
  must_bind!(sep, "reward_id" = reward_id);
  query.push(" RETURNING updated_at");

  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a campaign__reward
pub async fn unlink(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  reward_id: Uuid,
) -> Result<(), Error> {
  match query!(
    "DELETE FROM campaign__reward
    WHERE org_id = $1
      AND campaign_id = $2
      AND reward_id = $3",
    org_id,
    campaign_id,
    reward_id
  )
  .execute(db)
  .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}
