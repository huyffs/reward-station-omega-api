use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::Deserialize;

use crate::{
  // api::cm::org::MAX_PERMISSION,
  auth::{user::UserService, CustomClaims, MyFirebaseUser},
};

// pub async fn init(user: MyFirebaseUser, State(mut claims_service): State<UserService>) -> Response {
//   match claims_service
//     .set_custom_attributes(
//       &user.sub,
//       CustomClaims {
//         admin: MAX_PERMISSION,
//         orgs: user.claims.orgs,
//         wallets: user.claims.wallets,
//       },
//     )
//     .await
//   {
//     Ok(()) => StatusCode::OK.into_response(),
//     Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
//   }
// }

#[derive(Deserialize)]
pub struct GrantParams {
  pub user_id: String,
  pub level: i64,
}

pub async fn grant(
  user: MyFirebaseUser,
  State(mut claims_service): State<UserService>,
  Json(p): Json<GrantParams>,
) -> Response {
  if user.claims.admin < p.level {
    return StatusCode::FORBIDDEN.into_response();
  }

  match claims_service
    .set_custom_attributes(
      &p.user_id,
      CustomClaims {
        admin: p.level,
        orgs: user.claims.orgs,
        wallets: user.claims.wallets,
      },
    )
    .await
  {
    Ok(()) => StatusCode::OK.into_response(),
    Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
  }
}
