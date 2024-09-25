use axum::{
  extract::{Path, Query, State},
  response::Response,
};
use uuid::Uuid;

use crate::{
  api::handle_result,
  db::{self, CreatedFilter},
};

pub async fn get(State(db): State<sqlx::PgPool>, Path(project_id): Path<Uuid>) -> Response {
  let res = db::project_pub::get(&db, project_id);

  handle_result(res.await)
}

pub async fn list(State(db): State<sqlx::PgPool>, Query(p): Query<db::ListParams::<CreatedFilter>>) -> Response {
  let res = db::project_pub::list(
    &db,
    p,
  );

  handle_result(res.await)
}
