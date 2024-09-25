use std::{collections::HashMap, str::FromStr};

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use ethers::types::Address;

use crate::auth::{user::UserService, CustomClaims, MyFirebaseUser};

pub async fn fix_claims(
  user: MyFirebaseUser,
  State(mut claims_service): State<UserService>,
) -> Result<StatusCode, Response> {
  let mut my_wallets = HashMap::<String, bool>::new();
  user.claims.wallets.keys().for_each(|k| {
    let v = user.claims.wallets.get(k);
    if let Some(true) = v {
      my_wallets.insert(k.to_lowercase(), true);
    }
  });

  match claims_service
    .set_custom_attributes(
      &user.sub,
      CustomClaims {
        admin: user.claims.admin,
        orgs: user.claims.orgs,
        wallets: my_wallets,
      },
    )
    .await
  {
    Ok(()) => Ok(StatusCode::OK),
    Err(err) => {
      tracing::error!("Error set claims: {}", err);
      Err(StatusCode::BAD_GATEWAY.into_response())
    }
  }
}

#[derive(Debug, serde::Deserialize)]
pub struct ConnectParams {
  sig: String,
  user_id: String,
  chain_id: i64,
  signer_address: String,
}

pub async fn link(
  user: MyFirebaseUser,
  State(mut claims_service): State<UserService>,
  Json(param): Json<ConnectParams>,
) -> Result<StatusCode, Response> {
  let signer_address = param.signer_address.to_lowercase();
  if user.sub != param.user_id {
    return Err(StatusCode::FORBIDDEN.into_response());
  }
  let sig = ethers::types::Signature::from_str(&param.sig)
    .map_err(|_| StatusCode::BAD_REQUEST.into_response())?;
  let msg = format!(
    "{}/{}/{}",
    param.user_id, param.chain_id, signer_address
  );
  let addr = Address::from_str(&signer_address)
    .map_err(|_| StatusCode::BAD_REQUEST.into_response())?;
  sig
    .verify(msg, addr)
    .map_err(|_| StatusCode::FORBIDDEN.into_response())?;
  let mut my_wallets = user.claims.wallets;
  my_wallets.insert(
    format!("{}/{}", param.chain_id, signer_address),
    true,
  );
  match claims_service
    .set_custom_attributes(
      &user.sub,
      CustomClaims {
        admin: user.claims.admin,
        orgs: user.claims.orgs,
        wallets: my_wallets,
      },
    )
    .await
  {
    Ok(()) => Ok(StatusCode::OK),
    Err(err) => {
      tracing::error!("Error set claims: {}", err);
      Err(StatusCode::BAD_GATEWAY.into_response())
    }
  }
}
