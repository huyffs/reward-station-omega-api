use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use axum_extra::extract::Query;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_with::{serde_as, NoneAsEmptyString};
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{
    self, new_uuid,
    reward::{CreateParam, ReplaceParams, RewardFilter, UpdateParams},
    IdCreateResult, IdPrefix,
  },
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  Path(id): Path<Uuid>,
) -> Response {
  let res = db::reward::get(&db, id);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  Query(p): Query<db::ListParams<RewardFilter>>,
) -> Response {
  let res = db::reward::list(&db, p);

  handle_result(res.await)
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct CreateForm {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub description: Option<String>,
  pub tandc: Option<String>,
  pub images: Option<Vec<String>>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_until: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_until: Option<NaiveDate>,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Json(p): Json<CreateForm>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let id = new_uuid(IdPrefix::Reward);
  let res = db::reward::create(
    &db,
    CreateParam {
      id,
      images: p.images.unwrap_or_default(),
      issuer_id: p.issuer_id,
      category: p.category,
      country_id: p.country_id,
      name: p.name,
      description: p.description,
      tandc: p.tandc,
      active_from: p.active_from,
      active_until: p.active_until,
      valid_from: p.valid_from,
      valid_until: p.valid_until,
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
  Path(id): Path<Uuid>,
  Json(p): Json<UpdateParams>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::reward::update(&db, id, p);

  handle_result(res.await)
}

pub async fn replace(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(id): Path<Uuid>,
  Json(p): Json<ReplaceParams>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::reward::replace(&db, id, p);

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(id): Path<Uuid>,
) -> Response {
  if !user.can_sudo() {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::reward::delete(&db, id).await {
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    _ => StatusCode::ACCEPTED.into_response(),
  }
}
