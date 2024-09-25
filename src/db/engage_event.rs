use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::{handle_pg_error, sqlx_macro::offset_limit};

use super::{engage::Submissions, maybe_order_by, Error, ListParams, Never};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct EngageEventLog {
  pub id: i64,
  pub org_id: Uuid,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub user_id: String,
  pub old_submissions: Option<Json<Submissions>>,
  pub old_accepted: Option<Json<HashMap<String, bool>>>,
  pub old_coupon_serial: Option<String>,
  pub old_coupon_url: Option<String>,
  pub new_submissions: Option<Json<Submissions>>,
  pub new_accepted: Option<Json<HashMap<String, bool>>>,
  pub new_coupon_serial: Option<String>,
  pub new_coupon_url: Option<String>,
  pub created_at: DateTime<Utc>,
}

pub async fn list(db: &PgPool, p: ListParams<Never>) -> Result<Vec<EngageEventLog>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      id,
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
      old_submissions,
      old_accepted,
      old_coupon_serial,
      old_coupon_url,
      new_submissions,
      new_accepted,
      new_coupon_serial,
      new_coupon_url,
      created_at
    FROM engage_event"#,
  );

  maybe_order_by(&mut query, &p.order, vec!["created_at"])?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}
