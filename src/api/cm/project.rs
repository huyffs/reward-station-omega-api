use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  api::handle_result,
  auth::MyFirebaseUser,
  db::{
    self, new_uuid,
    project::{CreateParam, Networks, ProjectFilter, ReplaceParams, UpdateParams},
    IdCreateResult, IdPrefix,
  },
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::project::get(&db, org_id, project_id);

  handle_result(res.await)
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct ListFilter {
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<ListFilter>>,
  Path(org_id): Path<Uuid>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::project::list(
    &db,
    db::ListParams::<ProjectFilter> {
      filter: ProjectFilter {
        org_id,
        created_after: p.filter.created_after,
        created_before: p.filter.created_before,
      },
      order: p.order,
      offset: p.offset,
      limit: p.limit,
    },
  );

  handle_result(res.await)
}

#[derive(Deserialize, Debug)]
pub struct CreateForm {
  pub name: String,
  pub logo: Option<String>,
  pub images: Option<Vec<String>>,
  pub website: String,
  pub networks: Option<Networks>,
  pub description: String,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path(org_id): Path<Uuid>,
  Json(p): Json<CreateForm>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let id = new_uuid(IdPrefix::Project);
  let res = db::project::create(
    &db,
    CreateParam {
      org_id,
      id,
      name: p.name,
      logo: p.logo,
      images: p.images.unwrap_or_default(),
      website: p.website,
      networks: p.networks.unwrap_or_default(),
      description: p.description,
    },
  );

  handle_result(res.await.map(|r| IdCreateResult {
    id,
    created_at: r.created_at,
  }))
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
  Json(p): Json<UpdateParams>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let params = if user.can_sudo() {
    UpdateParams {
      name: p.name,
      logo: p.logo,
      images: p.images,
      description: p.description,
      website: p.website,
      networks: p.networks,
      feature_from: p.feature_from,
      feature_until: p.feature_until,
    }
  } else {
    UpdateParams {
      name: p.name,
      logo: p.logo,
      images: p.images,
      description: p.description,
      website: p.website,
      networks: p.networks,
      feature_from: None,
      feature_until: None,
    }
  };

  let res = db::project::update(
    &db,
    org_id,
    project_id,
    params,
  );

  handle_result(res.await)
}

pub async fn replace(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
  Json(p): Json<ReplaceParams>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let params = if user.can_sudo() {
    ReplaceParams {
      name: p.name,
      logo: p.logo,
      images: p.images,
      description: p.description,
      website: p.website,
      networks: p.networks,
      feature_from: p.feature_from,
      feature_until: p.feature_until,
    }
  } else {
    ReplaceParams {
      name: p.name,
      logo: p.logo,
      images: p.images,
      description: p.description,
      website: p.website,
      networks: p.networks,
      feature_from: None,
      feature_until: None,
    }
  };

  let res = db::project::replace(&db, org_id, project_id, params);

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, project_id)): Path<(Uuid, Uuid)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  match db::project::delete(&db, org_id, project_id).await {
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    _ => StatusCode::ACCEPTED.into_response(),
  }
}
