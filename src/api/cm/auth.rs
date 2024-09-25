use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
  auth::{user::UserService, CustomClaims, MyFirebaseUser},
  db,
};

pub async fn grant(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  State(mut claims_service): State<UserService>,
  Path(org_id): Path<Uuid>,
) -> Response {
  let res = db::org::get(&db, org_id).await;

  match res {
    Err(db::Error::NotFound) => StatusCode::NOT_FOUND.into_response(),
    Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    Ok(org) => {
      let a_id = org_id.to_string();
      let admins = org.admins.0;
      let grant_permission = admins.get(&user.sub);
      if let Some(g) = grant_permission {
        match user.claims.orgs.get(&org_id.to_string()) {
          Some(p) if p == g => StatusCode::NOT_MODIFIED.into_response(),
          _ => {
            let mut new_orgs = user.claims.orgs.to_owned();
            new_orgs.insert(a_id, g.to_owned());
            match claims_service
              .set_custom_attributes(
                &user.sub,
                CustomClaims {
                  admin: user.claims.admin,
                  orgs: new_orgs,
                  wallets: user.claims.wallets,
                },
              )
              .await
            {
              Ok(()) => StatusCode::OK.into_response(),
              Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
            }
          }
        }
      } else {
        StatusCode::FORBIDDEN.into_response()
      }
    }
  }
}
