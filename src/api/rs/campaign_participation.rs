use axum::{
  extract::{Path, State},
  response::Response,
};
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db,
};

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(campaign_id): Path<Uuid>,
) -> Response {
  let res = db::campaign_participation::list(&db, &user.sub, campaign_id);

  handle_result(res.await)
}
