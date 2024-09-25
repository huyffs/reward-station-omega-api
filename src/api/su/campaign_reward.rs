use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api::handle_db_error, auth::MyFirebaseUser, db};

#[derive(Deserialize, Default, Debug)]
pub struct UpdateParam {
  pub approved: bool,
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaignt_id, reward_id)): Path<(Uuid, Uuid, Uuid)>,
  Json(p): Json<UpdateParam>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::campaign_reward::update(
    &db,
    org_id,
    campaignt_id,
    reward_id,
    db::campaign_reward::UpdateParam {
      point: None,
      active: None,
      approved: Some(p.approved),
      max_mint: None,
      user_mint: None,
    },
  )
  .await
  {
    Err(err) => handle_db_error(err),
    _ => StatusCode::OK.into_response(),
  }
}
