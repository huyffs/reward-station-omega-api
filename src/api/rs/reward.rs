use axum::{
  extract::{Path, Query, State},
  response::Response,
};
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{self, reward::RewardFilter},
};

pub async fn get(State(db): State<sqlx::PgPool>, Path(reward_id): Path<Uuid>) -> Response {
  let res = db::reward::get(&db, reward_id);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  Query(p): Query<db::ListParams<RewardFilter>>,
) -> Response {
  let res = db::reward::list(&db, p);

  handle_result(res.await)
}

pub async fn mint_from_project(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((project_id, reward_id)): Path<(Uuid, Uuid)>,
) -> Response {
  let res = db::reward::mint_project_reward(&db, &user.sub, reward_id, project_id);

  handle_result(res.await)
}

pub async fn mint_from_campaign(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((campaign_id, reward_id)): Path<(Uuid, Uuid)>,
) -> Response {
  let res = db::reward::mint_campaign_reward(&db, &user.sub, reward_id, campaign_id);

  handle_result(res.await)
}
