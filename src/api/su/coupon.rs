use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  api::{handle_db_error, handle_result},
  auth::MyFirebaseUser,
  db::{
    self,
    coupon::{Coupon, CouponFilter, ReplaceParams, UpdateParams},
  },
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((reward_id, number)): Path<(Uuid, i64)>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::coupon::get(&db, reward_id, number);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<CouponFilter>>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::coupon::list(&db, p);

  handle_result(res.await)
}

#[derive(Deserialize, Debug)]
pub struct CreateForm {
  pub reward_id: Uuid,
  pub urls: Vec<String>,
}

#[derive(Serialize)]
pub struct CreateResult {
  pub coupons: Vec<Coupon>,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Json(p): Json<CreateForm>,
) -> Result<Json<CreateResult>, Response> {
  if !user.can_sudo() {
    return Err(StatusCode::FORBIDDEN.into_response());
  }

  let number = db::coupon::next_seq(&db, p.reward_id)
    .await
    .map_err(handle_db_error)?;

  let coupons = db::coupon::create_batch(&db, p.reward_id, number, p.urls)
    .await
    .map_err(handle_db_error)?;

  Ok(Json(CreateResult { coupons }))
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((reward_id, number)): Path<(Uuid, i64)>,
  Json(p): Json<UpdateParams>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::coupon::update(&db, reward_id, number, p);

  handle_result(res.await)
}

pub async fn replace(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((reward_id, number)): Path<(Uuid, i64)>,
  Json(p): Json<ReplaceParams>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::coupon::replace(&db, reward_id, number, p);

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((reward_id, number)): Path<(Uuid, i64)>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::coupon::delete(&db, reward_id, number).await {
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    _ => StatusCode::ACCEPTED.into_response(),
  }
}
