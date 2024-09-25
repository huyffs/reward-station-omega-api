use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, maybe_bind, offset_limit};

use super::{handle_pg_error, maybe_order_by, Error};

#[derive(FromRow, Serialize)]
pub struct Coupon {
  pub reward_id: Uuid,
  pub number: i64,
  pub url: String,
  pub minted_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ListFilter {
  pub reward_id: Option<String>,
  pub minted_before: Option<DateTime<Utc>>,
  pub minted_after: Option<DateTime<Utc>>,
}

// list coupons
pub async fn list(
  db: &PgPool,
  user_id: &str,
  p: super::ListParams<ListFilter>,
) -> Result<Vec<Coupon>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      reward_id,
      number,
      url,
      minted_at
    FROM coupon"#,
  );

  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "user_id" = user_id);
  maybe_bind!(sep, "reward_id" = p.filter.reward_id);
  maybe_bind!(sep, "minted_at" <= p.filter.minted_before);
  maybe_bind!(sep, "minted_at" >= p.filter.minted_after);

  maybe_order_by(&mut query, &p.order, vec!["minted_at"])?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a coupon
pub async fn get(
  db: &PgPool,
  user_id: &str,
  reward_id: Uuid,
  number: i64,
) -> Result<Coupon, Error> {
  query_as!(
    Coupon,
    r#"SELECT
      reward_id,
      number,
      url,
      minted_at
    FROM coupon
    WHERE reward_id = $1
      AND number = $2
      AND user_id = $3"#,
    reward_id,
    number,
    user_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}
