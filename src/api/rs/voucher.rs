use axum::{
  extract::{Path, State},
  response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use http::StatusCode;
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{self, voucher::ListFilter},
};

pub async fn get(
  user: MyFirebaseUser,
  State(db): State<sqlx::PgPool>,
  Path((campaign_id, chain_id, signer_address, task_id)): Path<(Uuid, i64, String, String)>,
) -> Response {
  let signer_address = signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, &signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::voucher::get(&db, campaign_id, chain_id, signer_address, task_id);

  handle_result(res.await)
}

pub async fn list(
  user: MyFirebaseUser,
  State(db): State<sqlx::PgPool>,
  Query(p): Query<db::ListParams<ListFilter>>,
) -> Response {
  let res = db::voucher::list(&db, &user.sub, p);

  handle_result(res.await)
}

pub async fn get_project_point(
  user: MyFirebaseUser,
  State(db): State<sqlx::PgPool>,
  Path(project_id): Path<Uuid>,
) -> Response {
  let res = db::voucher::get_project_point(&db, &user.sub, project_id);

  handle_result(res.await)
}
