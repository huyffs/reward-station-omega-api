use axum::{
  extract::{Path, State},
  response::Response,
};
use axum_extra::extract::Query;
use uuid::Uuid;

use crate::{api::handle_result, auth::MyFirebaseUser, db};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((reward_id, number)): Path<(Uuid, i64)>,
) -> Response {
  let res = db::coupon_pub::get(&db, &user.sub, reward_id, number);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<db::coupon_pub::ListFilter>>,
) -> Response {
  let res = db::coupon_pub::list(&db, &user.sub, p);

  handle_result(res.await)
}
