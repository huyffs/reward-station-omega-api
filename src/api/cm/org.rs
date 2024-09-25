use std::collections::HashMap;

use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::{user::UserService, MyFirebaseUser, OWNER_PERMISSION},
  db::{
    self, new_uuid,
    org::{CreateForm, ReplaceParams, UpdateParam},
    CreateParam, CreatedFilter, IdPrefix,
  },
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::org::get(&db, org_id);

  handle_result(res.await)
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<CreatedFilter>>,
) -> Response {
  let res = db::org::list(&db, &user.sub, p);

  handle_result(res.await)
}

#[derive(Serialize)]
pub struct OrgCreated {
  id: Uuid,
  admins: HashMap<String, i64>,
  created_at: DateTime<Utc>,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  State(mut claims_service): State<UserService>,
  Json(mut form): Json<CreateForm>,
) -> Response {
  let id = new_uuid(IdPrefix::Org);
  let permission = OWNER_PERMISSION;
  let mut claims = user.claims;
  claims.orgs.insert(id.to_string(), permission);

  match claims_service
    .set_custom_attributes(&user.sub, claims)
    .await
  {
    Ok(()) => {
      form.admins.insert(user.sub, permission);

      let res = db::org::create(
        &db,
        CreateParam {
          id,
          form: &form,
        },
      )
      .await;

      handle_result(res.map(move |r| OrgCreated {
        id,
        admins: form.admins,
        created_at: r.created_at,
      }))
    }
    Err(err) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      format!("Error update claims: {}", err),
    )
      .into_response(),
  }
}

#[derive(Deserialize)]
pub struct UpdateForm {
  pub name: Option<String>,
  pub logo: Option<String>,
  pub admins: Option<HashMap<String, i64>>,
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
  Json(form): Json<UpdateForm>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::org::update(
    &db,
    org_id,
    UpdateParam {
      name: form.name,
      logo: form.logo,
      admins: form.admins,
    },
  );

  handle_result(res.await)
}

pub async fn replace(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
  Json(p): Json<ReplaceParams>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::org::replace(&db, org_id, p);

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::org::delete(&db, org_id).await {
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    _ => StatusCode::ACCEPTED.into_response(),
  }
}
