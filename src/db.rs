use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

pub mod campaign;
pub mod campaign_participation;
pub mod campaign_reward;
pub mod campaign_reward_pub;
pub mod coupon;
pub mod coupon_pub;
pub mod engage;
pub mod engage_event;
pub mod mezzofy;
pub mod org;
pub mod project;
pub mod project_reward;
pub mod project_reward_pub;
pub mod campaign_pub;
pub mod engage_pub;
pub mod project_pub;
pub mod project_membership;
pub mod reward;
pub mod voucher;
pub mod me;
pub mod sqlx_macro;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Not found")]
  NotFound,
  #[error("Empty update set")]
  EmptyUpdateSet,
  #[error("Validation error")]
  Validation,
  #[error("Invalid order param")]
  InvalidOrder,
  #[error("Unknown error")]
  LimitReached,
  #[error("Limit reached")]
  Unknown,
  #[error("Unknown sqlx error {0}")]
  Sqlx(#[from] sqlx::Error),
}

#[repr(u8)]
pub enum IdPrefix {
  Org = 0x01,
  Project = 0x02,
  Campaign = 0x03,
  Reward = 0x04,
  // IdempotentKey=0xFF,
}

pub fn new_uuid(kind: IdPrefix) -> Uuid {
  let id = Uuid::new_v4();
  let mut b = id.to_bytes_le();
  b[0] = kind as u8;
  Uuid::from_bytes_le(b)
}

#[derive(Deserialize, Clone)]
pub struct Never {}

impl IsEmpty for Never {
  fn is_empty(&self) -> bool {
    true
  }
}

#[derive(Deserialize, Default, Debug)]
pub struct ListParams<T: Clone + Send> {
  #[serde(flatten)]
  pub filter: T,
  #[serde(rename = "_s")]
  pub order: Option<String>,
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

#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct CreatedFilter {
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

pub fn get_order_by_sql(order: &str, cols: Vec<&str>) -> Result<String, Error> {
  let s: String;
  let sort = if order.starts_with('-') {
    s = order.chars().skip(1).collect();
    "desc"
  } else {
    s = order.to_string();
    "asc"
  };
  for c in cols {
    if c == s {
      return Ok(format!("{} {}", c, sort));
    }
  }
  Err(Error::InvalidOrder)
}

pub fn maybe_order_by<'a>(
  query: &mut QueryBuilder<'a, Postgres>,
  value: &Option<String>,
  allow_cols: Vec<&'a str>,
) -> Result<(), Error> {
  if let Some(order) = value {
    let order = get_order_by_sql(order, allow_cols)?;
    query.push(" ORDER BY ");
    query.push(order);
  }
  Ok(())
}

pub fn handle_pg_error(err: sqlx::Error) -> Error {
  match err {
    sqlx::Error::RowNotFound => Error::NotFound,
    _ => Error::Sqlx(err),
  }
}

pub struct CreateParam<'a, T> {
  pub id: Uuid,
  pub form: &'a T,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct IdCreateResult<T: Serialize> {
  pub id: T,
  pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct CreateResult {
  pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct UpdateResult {
  pub updated_at: DateTime<Utc>,
}

// check health
pub async fn health(db: &sqlx::PgPool) -> Result<(), Error> {
  let res: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1").fetch_one(db).await;
  match res {
    Ok(row) if row.0 == 1 => Ok(()),
    Err(err) => Err(Error::Sqlx(err)),
    _ => Err(Error::Unknown),
  }
}
