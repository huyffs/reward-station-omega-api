use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use uuid::Uuid;

use crate::{
  api::{handle_db_error, handle_result},
  auth::MyFirebaseUser,
  db::{self, project_reward::ProjectRewardFilter},
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id, reward_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::project_reward::get(&db, org_id, project_id, reward_id);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
  Query(p): Query<db::ListParams<ProjectRewardFilter>>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::project_reward::list(&db, org_id, project_id, p);

  handle_result(res.await)
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
  Json(p): Json<db::project_reward::CreateParam>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::project_reward::create(&db, org_id, project_id, p);
  handle_result(res.await)
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id, reward_id)): Path<(Uuid, Uuid, Uuid)>,
  Json(p): Json<db::project_reward::UpdateParam>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::project_reward::update(
    &db,
    org_id,
    project_id,
    reward_id,
    db::project_reward::UpdateParam {
      approved: None,
      point: p.point,
      active: p.active,
      max_mint: Some(p.max_mint.unwrap_or_default()),
      user_mint: Some(p.user_mint.unwrap_or_default()),
    },
  );

  handle_result(res.await)
}

pub async fn unlink(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id, reward_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::project_reward::unlink(&db, org_id, project_id, reward_id).await {
    Err(err) => handle_db_error(err),
    _ => StatusCode::OK.into_response(),
  }
}
