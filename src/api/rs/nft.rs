use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use is_empty::IsEmpty;
use serde::Deserialize;

use crate::{api::into_json_response, auth::MyFirebaseUser, subscan};

#[derive(Deserialize, Default, Debug)]
pub struct ListParams<T: IsEmpty + Clone + Send> {
  #[serde(flatten)]
  pub filter: T,
  #[serde(rename = "_o")]
  #[serde(default = "default_offset")]
  pub offset: u64,
  #[serde(rename = "_l")]
  #[serde(default = "default_limit")]
  pub limit: u64,
}

pub fn default_offset() -> u64 {
  0u64
}

pub fn default_limit() -> u64 {
  10000u64
}

#[derive(Debug, IsEmpty, Clone, serde::Deserialize)]
pub struct ListFilter {
  pub contract_addresses: Option<Vec<String>>,
}

pub async fn list(
  user: MyFirebaseUser,
  State(subscan_client): State<subscan::Client>,
  // Query(p): Query<ListParams<ListFilter>>,
  Path((chain_id, signer_address)): Path<(i64, String)>,
) -> Response {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = subscan_client.get_nfts_for_owner(chain_id, signer_address);

  match res.await {
    Ok(nfts) => into_json_response(&nfts),
    Err(err) => (StatusCode::BAD_GATEWAY, err.to_string()).into_response(),
  }
}
