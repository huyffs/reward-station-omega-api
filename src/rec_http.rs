use std::collections::HashMap;

use crate::db::mezzofy::{CreateParam, CreateResult, UpdateParams};

use crate::db::{self, UpdateResult};
use anyhow::bail;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct Headers(pub HashMap<String, Vec<String>>);

impl Headers {
  pub fn from_request_headers(headers: &HeaderMap<HeaderValue>) -> Self {
    let mut header_hashmap = HashMap::<String, Vec<String>>::new();
    for (k, v) in headers {
      let k = k.as_str().to_owned();
      let v = String::from_utf8_lossy(v.as_bytes()).into_owned();
      header_hashmap.entry(k).or_default().push(v)
    }
    Self(header_hashmap)
  }
}

#[derive(Debug, Clone)]
pub struct Client {
  db: sqlx::PgPool,
  c: reqwest::Client,
}

impl<'a> Client {
  pub fn new(db: sqlx::PgPool) -> Self {
    Self {
      db,
      c: reqwest::Client::new(),
    }
  }

  pub async fn get(&self, url: &str, headers: HeaderMap) -> Result<Option<String>, anyhow::Error> {
    self
      .process::<()>(http::Method::GET, url, headers, None, self.c.get(url))
      .await
  }

  pub async fn post_json<T: Serialize>(
    &self,
    url: &str,
    headers: HeaderMap,
    data: &T,
  ) -> Result<Option<String>, anyhow::Error> {
    self
      .process(
        http::Method::POST,
        url,
        headers,
        Some(data),
        self.c.post(url).json(data),
      )
      .await
  }

  pub async fn post_form<T: Serialize>(
    &self,
    url: &str,
    headers: HeaderMap,
    data: &T,
  ) -> Result<Option<String>, anyhow::Error> {
    self
      .process(
        http::Method::POST,
        url,
        headers,
        Some(data),
        self.c.post(url).form(data),
      )
      .await
  }

  async fn process<T: Serialize>(
    &self,
    method: http::Method,
    url: &str,
    headers: HeaderMap,
    data: Option<&T>,
    req: reqwest::RequestBuilder,
  ) -> Result<Option<String>, anyhow::Error> {
    // let req_body = new_uuid(IdPrefix::IdempotentKey);

    let data = data.map(|t| serde_json::to_string(t).unwrap_or_default());
    let create_res = self
      .create(
        &self.db,
        method,
        url,
        &Headers::from_request_headers(&headers),
        data,
      )
      .await?;

    match req.headers(headers).send().await {
      Ok(res) => {
        let status = res.status();
        let res_headers = Headers::from_request_headers(res.headers());
        let res_body = res.text().await?;

        self
          .update(
            &self.db,
            create_res.id,
            Some(&res_headers),
            res_body.as_str(),
            status,
          )
          .await?;

        if status == StatusCode::OK {
          Ok(Some(res_body))
        } else {
          bail!(res_body)
        }
      }
      Err(err) => {
        let kind = if err.is_body() {
          "Body"
        } else if err.is_builder() {
          "Builder"
        } else if err.is_connect() {
          "Connect"
        } else if err.is_decode() {
          "Decode"
        } else if err.is_redirect() {
          "Redirect"
        } else if err.is_request() {
          "Request"
        } else if err.is_status() {
          "Status"
        } else if err.is_timeout() {
          "Timeout"
        } else {
          "Unknown"
        };
        let msg = format!("{} error: {}", kind, err);
        self
          .update(&self.db, create_res.id, None, &msg, StatusCode::IM_A_TEAPOT)
          .await?;
        bail!(msg)
      }
    }
  }

  async fn create(
    &self,
    db: &PgPool,
    method: http::Method,
    url: &str,
    headers: &Headers,
    body: Option<String>,
  ) -> Result<CreateResult, db::Error> {
    db::mezzofy::create(
      db,
      CreateParam {
        idemp_key: None,
        method: method.as_str(),
        url,
        req_headers: headers,
        req_body: body,
      },
    )
    .await
  }

  async fn update(
    &self,
    db: &PgPool,
    id: i64,
    headers: Option<&Headers>,
    body: &str,
    status: StatusCode,
  ) -> Result<UpdateResult, db::Error> {
    db::mezzofy::update(
      db,
      id,
      UpdateParams {
        res_headers: headers,
        res_body: match body.is_empty() {
          true => None,
          false => Some(body),
        },
        status: Some(status.as_u16() as i16),
      },
    )
    .await
  }
}
