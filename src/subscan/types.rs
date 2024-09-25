use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ListAccountTokenPayload<'a> {
  pub address: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
  pub code: u8,
  pub message: String,
  pub generated_at: u64,
  pub data: T,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ListAccountTokenResult {
  pub native: Vec<AstarToken>,
  pub ERC721: Vec<ERC721Token>,
}

#[derive(Debug, Deserialize)]
pub struct AstarToken {
  pub symbol: String,
  pub unique_id: String,
  pub decimals: u8,
  pub balance: String,
  pub lock: String,
  pub reserved: String,
  pub bonded: String,
  pub unbonding: String,
  pub democracy_lock: String,
  pub conviction_lock: String,
  pub election_lock: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ERC721Token {
  pub symbol: String,
  pub unique_id: String,
  pub decimals: u8,
  pub balance: String,
  pub contract: String,
}
