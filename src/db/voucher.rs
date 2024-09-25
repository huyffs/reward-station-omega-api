use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::{prelude::FromRow, query, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, maybe_bind, offset_limit};

use super::{handle_pg_error, maybe_order_by, Error};

#[derive(FromRow, Serialize)]
pub struct Voucher {
  pub org_id: Option<Uuid>,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub user_id: String,
  pub task_id: String,
  pub value: i64,
  pub balance: i64,
  pub valid_from: Option<NaiveDate>,
  pub valid_until: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

pub async fn get_project_point(
  db: &PgPool,
  user_id: &str,
  project_id: Uuid,
) -> Result<i64, Error> {
  let res = query!(
    r#"SELECT
      SUM(balance) AS "total!: i64"
    FROM voucher
    WHERE project_id = $1
      AND user_id = $2
      AND valid_from <= NOW()
      AND valid_until >= NOW()
      "#,
    project_id,
    user_id,
  )
  .fetch_one(db)
  .await;
  let point = match res {
    Ok(point) => Ok(point.total),
    Err(err) => match err {
      sqlx::Error::RowNotFound => Ok(0),
      _ => Err(Error::Sqlx(err)),
    },
  }?;

  Ok(point)
}

#[serde_as]
#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct ListFilter {
  pub org_id: Option<Uuid>,
  pub project_id: Option<Uuid>,
  pub campaign_id: Option<Uuid>,
  pub chain_id: Option<i64>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_before: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_after: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub created_after: Option<DateTime<Utc>>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub created_before: Option<DateTime<Utc>>,
}

// list vouchers
pub async fn list(
  db: &PgPool,
  user_id: &str,
  p: super::ListParams<ListFilter>,
) -> Result<Vec<Voucher>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
    org_id,
    project_id,
    campaign_id,
    chain_id,
    signer_address,
    user_id,
    task_id,
    value,
    balance,
    valid_from,
    valid_until,
    created_at,
    updated_at
    FROM voucher"#,
  );

  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "user_id" = user_id);
  maybe_bind!(sep, "org_id" = p.filter.org_id);
  maybe_bind!(sep, "project_id" = p.filter.project_id);
  maybe_bind!(sep, "campaign_id" = p.filter.campaign_id);
  maybe_bind!(sep, "chain_id" = p.filter.chain_id);
  maybe_bind!(sep, "valid_before" <= p.filter.valid_before);
  maybe_bind!(sep, "valid_after" >= p.filter.valid_after);
  maybe_bind!(sep, "created_at" <= p.filter.created_before);
  maybe_bind!(sep, "created_at" >= p.filter.created_after);

  maybe_order_by(
    &mut query,
    &p.order,
    vec!["valid_before", "valid_after", "created_at", "updated_at"],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a voucher
pub async fn get(
  db: &PgPool,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: String,
  task_id: String,
) -> Result<Voucher, Error> {
  query_as!(
    Voucher,
    r#"SELECT
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
      task_id,
      value,
      balance,
      valid_from,
      valid_until,
      created_at,
      updated_at
    FROM voucher
    WHERE campaign_id = $1
      AND chain_id = $2
      AND signer_address = $3
      AND task_id = $4"#,
    campaign_id,
    chain_id,
    signer_address,
    task_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}
