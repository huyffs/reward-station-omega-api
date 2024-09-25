use axum::{
  extract::{Query, State},
  response::Response, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{self, project_membership::ProjectMembershipFilter, ListParams},
};

pub async fn list_my_clubs(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<ListParams<ProjectMembershipFilter>>,
) -> Response {
  let res = db::project_membership::list(&db, &user.sub, p);

  handle_result(res.await)
}


#[derive(Deserialize, Debug)]
pub struct JoinForm {
  pub project_id: Uuid,
}

pub async fn join(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Json(form): Json<JoinForm>,
) -> Response {
  let res = db::project_membership::join(&db, form.project_id, &user.sub);

  handle_result(res.await)
}
