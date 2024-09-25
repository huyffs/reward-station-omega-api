use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{
  prelude::FromRow,
  query_as,
  types::{Decimal, Json},
  PgPool, Postgres, QueryBuilder,
};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, maybe_bind, offset_limit};

use super::{handle_pg_error, maybe_order_by, CreateResult, Error, UpdateResult};

#[derive(FromRow, Serialize)]
pub struct Campaign {
  pub org_id: Option<Uuid>,
  pub project_id: Uuid,
  pub id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: Option<String>,
  pub coupon_code: Option<String>,
  pub budget: Option<Decimal>,
  pub chain_id: i64,
  pub contract_address: String,
  pub condition_info: Option<String>,
  pub reward_amount: Option<Decimal>,
  pub reward_info: Option<String>,
  pub tasks: Json<Vec<Task>>,
  pub start_at: Option<NaiveDate>,
  pub end_at: Option<NaiveDate>,
  pub voucher_policy: i16,
  pub voucher_expire_at: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct Task {
  pub id: String,
  pub name: String,
  pub description: Option<String>,
  pub link: Option<String>,
  pub images: Option<Vec<String>>,
  pub point: Option<i64>,
}

#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct CampaignFilter {
  pub org_id: Option<Uuid>,
  pub project_id: Option<Uuid>,
  pub chain_id: Option<u64>,
  pub contracts: Option<Vec<String>>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

// list campaigns
pub async fn list(
  db: &PgPool,
  p: super::ListParams<CampaignFilter>,
) -> Result<Vec<Campaign>, Error> {
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
      reward_amount,
      reward_info,
      tasks,
      start_at,
      end_at,
      voucher_policy,
      voucher_expire_at,
      created_at,
      updated_at
    FROM campaign"#,
  );

  if !p.filter.is_empty() {
    query.push(" WHERE ");
    let mut sep = query.separated(" AND ");
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
  }

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
pub async fn get(db: &PgPool, org_id: Uuid, campaign_id: Uuid) -> Result<Campaign, Error> {
  query_as!(
    Campaign,
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
    reward_amount,
    reward_info,
    tasks AS "tasks: Json<Vec<Task>>",
    start_at,
    end_at,
    voucher_policy,
    voucher_expire_at,
  created_at,
    updated_at
  FROM campaign
    WHERE org_id = $1 AND id = $2"#,
    org_id,
    campaign_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

pub struct CreateParam {
  pub org_id: Uuid,
  pub project_id: Uuid,
  pub id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: String,
  pub coupon_code: Option<String>,
  pub budget: Option<Decimal>,
  pub chain_id: i64,
  pub contract_address: String,
  pub condition_info: Option<String>,
  pub reward_amount: Option<Decimal>,
  pub reward_info: Option<String>,
  pub tasks: Vec<Task>,
  pub start_at: Option<NaiveDate>,
  pub end_at: Option<NaiveDate>,
  pub voucher_policy: i16,
  pub voucher_expire_at: Option<NaiveDate>,
}

// create a campaign
pub async fn create(db: &PgPool, p: CreateParam) -> Result<CreateResult, Error> {
  query_as!(
    CreateResult,
    "INSERT INTO campaign (
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
      reward_amount,
      reward_info,
      tasks,
      start_at,
      end_at,
      voucher_policy,
      voucher_expire_at
      )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
    RETURNING created_at",
    p.org_id,
    p.project_id,
    p.id,
    p.name,
    p.logo,
    &p.images,
    p.description,
    p.coupon_code,
    p.budget,
    p.chain_id,
    p.contract_address,
    p.condition_info,
    p.reward_amount,
    p.reward_info,
    Json(&p.tasks) as _,
    p.start_at,
    p.end_at,
    p.voucher_policy,
    p.voucher_expire_at,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[derive(Deserialize, IsEmpty)]
pub struct UpdateParams {
  pub name: Option<String>,
  pub logo: Option<String>,
  pub images: Option<Vec<String>>,
  pub description: Option<String>,
  pub coupon_code: Option<String>,
  pub budget: Option<Decimal>,
  pub chain_id: Option<u64>,
  pub contract_address: Option<String>,
  pub condition_info: Option<String>,
  pub reward_amount: Option<Decimal>,
  pub reward_info: Option<String>,
  pub tasks: Option<Vec<Task>>,
  pub start_at: Option<NaiveDate>,
  pub end_at: Option<NaiveDate>,
  pub created_at: Option<DateTime<Utc>>,
  pub updated_at: Option<DateTime<Utc>>,
  pub voucher_policy: Option<i16>,
  pub voucher_expire_at: Option<NaiveDate>,
}

// update a campaign
pub async fn update(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  p: UpdateParams,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE campaign SET");
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "name" = p.name);
  maybe_bind!(sep, "logo" = p.logo);
  maybe_bind!(sep, "images" = p.images);
  maybe_bind!(sep, "description" = p.description);
  maybe_bind!(sep, "coupon_code" = p.coupon_code);
  maybe_bind!(sep, "budget" = p.budget);
  maybe_bind!(sep, "chain_id" = p.chain_id.map(|v| v as i64));
  maybe_bind!(sep, "contract_address" = p.contract_address);
  maybe_bind!(sep, "condition_info" = p.condition_info);
  maybe_bind!(sep, "reward_amount" = p.reward_amount);
  maybe_bind!(sep, "reward_info" = p.reward_info);
  maybe_bind!(sep, "tasks", p.tasks, Json);
  maybe_bind!(sep, "start_at" = p.start_at);
  maybe_bind!(sep, "end_at" = p.end_at);
  maybe_bind!(sep, "voucher_policy" = p.voucher_policy);
  maybe_bind!(sep, "voucher_expire_at" = p.voucher_expire_at);
  query.push(" WHERE org_id = ").push_bind(org_id);
  query.push(" AND id = ").push_bind(campaign_id);
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
  pub logo: Option<String>,
  pub images: Vec<String>,
  pub description: String,
  pub coupon_code: Option<String>,
  pub budget: Option<Decimal>,
  pub chain_id: i64,
  pub contract_address: String,
  pub condition_info: Option<String>,
  pub reward_amount: Option<Decimal>,
  pub reward_info: Option<String>,
  pub tasks: Vec<Task>,
  pub start_at: Option<NaiveDate>,
  pub end_at: Option<NaiveDate>,
  pub voucher_policy: i16,
  pub voucher_expire_at: Option<NaiveDate>,
}

// replace a campaign
pub async fn replace(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  p: ReplaceParams,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE campaign SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "name" = p.name);
  must_bind!(sep, "logo" = p.logo);
  must_bind!(sep, "images" = p.images);
  must_bind!(sep, "description" = p.description);
  must_bind!(sep, "coupon_code" = p.coupon_code);
  must_bind!(sep, "budget" = p.budget);
  must_bind!(sep, "chain_id" = p.chain_id);
  must_bind!(sep, "contract_address" = p.contract_address);
  must_bind!(sep, "condition_info" = p.condition_info);
  must_bind!(sep, "reward_amount" = p.reward_amount);
  must_bind!(sep, "reward_info" = p.reward_info);
  must_bind!(sep, "tasks" = Json(p.tasks));
  must_bind!(sep, "start_at" = p.start_at);
  must_bind!(sep, "end_at" = p.end_at);
  must_bind!(sep, "voucher_policy" = p.voucher_policy);
  must_bind!(sep, "voucher_expire_at" = p.voucher_expire_at);
  query.push(" WHERE org_id = ").push_bind(org_id);
  query.push(" AND id = ").push_bind(campaign_id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a campaign
pub async fn delete(db: &PgPool, org_id: Uuid, campaign_id: Uuid) -> Result<(), Error> {
  match sqlx::query!(
    "DELETE FROM campaign WHERE org_id = $1 AND id = $2",
    org_id,
    campaign_id
  )
  .execute(db)
  .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}
