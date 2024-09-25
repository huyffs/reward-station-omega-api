use reqwest::StatusCode;

mod types;

use self::types::{ERC721Token, ListAccountTokenPayload, ListAccountTokenResult, Response};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Unsupported chain")]
  UnsupportedChain,
  #[error("Reqwest error: {0}")]
  Reqwest(#[from] reqwest::Error),
  #[error("Deserialize error: {0}")]
  Deserialize(#[from] serde_json::Error),
  #[error("Subscan error: {status:?} {text:?}")]
  Subscan { status: StatusCode, text: String },
}

#[derive(Clone, Debug)]
pub struct Client {
  api_key: String,
  http_client: reqwest::Client,
}

impl Client {
  pub fn new(api_key: String, http_client: reqwest::Client) -> Self {
    Self {
      api_key,
      http_client,
    }
  }

  pub async fn get_nfts_for_owner(
    &self,
    chain_id: i64,
    address: &str,
  ) -> Result<Vec<ERC721Token>, Error> {
    let api_endpoint: &str = match chain_id {
      592 => Ok("https://astar.api.subscan.io/api"),
      _ => Err(Error::UnsupportedChain),
    }?;

    let res = self
      .http_client
      .post(format!("{}/scan/account/tokens", api_endpoint))
      .header("X-API-Key", &self.api_key)
      .json(&ListAccountTokenPayload { address })
      .send()
      .await
      .map_err(Error::Reqwest)?;

    let status = res.status();
    let text = res.text().await?;
    match status {
      StatusCode::OK => {
        let result: Response<ListAccountTokenResult> =
          serde_json::from_str(&text).map_err(Error::Deserialize)?;
        Ok(result.data.ERC721)
      }
      _ => Err(Error::Subscan { status, text }),
    }
  }
}
