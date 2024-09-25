use axum::{
  extract::{Path, Query, State},
  response::Response,
};
use uuid::Uuid;

use crate::{
  api::handle_result,
  db::{self, project_reward_pub::RewardFilter},
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  Path((project_id, reward_id)): Path<(Uuid, Uuid)>,
) -> Response {
  let res = db::project_reward_pub::get(&db, project_id, reward_id);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  Path(project_id): Path<Uuid>,
  Query(p): Query<db::ListParams<RewardFilter>>,
) -> Response {
  let res = db::project_reward_pub::list(&db, project_id, p);

  handle_result(res.await)
}
