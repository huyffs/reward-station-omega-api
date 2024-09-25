use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{
    self,
    campaign::{CampaignFilter, CreateParam, ReplaceParams, Task, UpdateParams},
    new_uuid, IdCreateResult, IdPrefix,
  },
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id)): Path<(Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::campaign::get(&db, org_id, campaign_id);

  handle_result(res.await)
}

#[derive(Deserialize, IsEmpty, Clone, Debug)]
pub struct ListFilter {
  pub project_id: Option<Uuid>,
  pub chain_id: Option<u64>,
  pub contracts: Option<Vec<String>>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<ListFilter>>,
  Path(org_id): Path<Uuid>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::campaign::list(
    &db,
    db::ListParams::<CampaignFilter> {
      filter: CampaignFilter {
        org_id: Some(org_id),
        project_id: p.filter.project_id,
        chain_id: p.filter.chain_id,
        contracts: p.filter.contracts,
        created_after: p.filter.created_after,
        created_before: p.filter.created_before,
      },
      order: p.order,
      offset: p.offset,
      limit: p.limit,
    },
  );

  handle_result(res.await)
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct CreateForm {
  pub project_id: Uuid,
  pub name: String,
  pub logo: Option<String>,
  pub images: Option<Vec<String>>,
  pub description: String,
  pub coupon_code: Option<String>,
  pub budget: Option<Decimal>,
  pub chain_id: i64,
  pub contract_address: String,
  pub condition_info: Option<String>,
  pub reward_amount: Option<Decimal>,
  pub reward_info: Option<String>,
  pub tasks: Vec<Task>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub start_at: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub end_at: Option<NaiveDate>,
  pub voucher_policy: Option<i16>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub voucher_expire_at: Option<NaiveDate>,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
  Json(p): Json<CreateForm>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let id = new_uuid(IdPrefix::Campaign);
  let res = db::campaign::create(
    &db,
    CreateParam {
      org_id,
      project_id: p.project_id,
      id,
      name: p.name,
      logo: p.logo,
      images: p.images.unwrap_or_default(),
      description: p.description,
      coupon_code: p.coupon_code,
      budget: p.budget,
      chain_id: p.chain_id,
      contract_address: p.contract_address.to_lowercase(),
      condition_info: p.condition_info,
      reward_amount: p.reward_amount,
      reward_info: p.reward_info,
      tasks: p.tasks,
      start_at: p.start_at,
      end_at: p.end_at,
      voucher_policy: p.voucher_policy.unwrap_or(1),
      voucher_expire_at: p.voucher_expire_at,
    },
  );

  handle_result(res.await.map(|r| IdCreateResult {
    id,
    created_at: r.created_at,
  }))
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id)): Path<(Uuid, Uuid)>,
  Json(p): Json<UpdateParams>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::campaign::update(&db, org_id, campaign_id, p);

  handle_result(res.await)
}

pub async fn replace(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id)): Path<(Uuid, Uuid)>,
  Json(p): Json<ReplaceParams>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::campaign::replace(&db, org_id, campaign_id, p);

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id)): Path<(Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::campaign::delete(&db, org_id, campaign_id).await {
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    _ => StatusCode::ACCEPTED.into_response(),
  }
}
