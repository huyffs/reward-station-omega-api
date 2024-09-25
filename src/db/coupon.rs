use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, maybe_bind, offset_limit};

use super::{handle_pg_error, maybe_order_by, Error, UpdateResult};

#[derive(FromRow, Serialize)]
pub struct Coupon {
  pub reward_id: Uuid,
  pub number: i64,
  pub url: String,
  pub user_id: Option<String>,
  pub minted_at: Option<DateTime<Utc>>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct CouponFilter {
  pub reward_id: Option<Uuid>,
  pub user_id: Option<String>,
  pub minted_before: Option<DateTime<Utc>>,
  pub minted_after: Option<DateTime<Utc>>,
}

// list coupons
pub async fn list(db: &PgPool, p: super::ListParams<CouponFilter>) -> Result<Vec<Coupon>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      reward_id,
      number,
      url,
      user_id,
      minted_at,
      created_at,
      updated_at
    FROM coupon"#,
  );

  if !p.filter.is_empty() {
    query.push(" WHERE ");
    let mut sep = query.separated(" AND ");
    maybe_bind!(sep, "reward_id" = p.filter.reward_id);
    maybe_bind!(sep, "user_id" = p.filter.user_id);
    maybe_bind!(sep, "minted_at" > p.filter.minted_before);
    maybe_bind!(sep, "minted_at" < p.filter.minted_after);
  }

  maybe_order_by(
    &mut query,
    &p.order,
    vec!["minted_before", "minted_after", "created_at", "updated_at"],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a coupon
pub async fn get(db: &PgPool, reward_id: Uuid, number: i64) -> Result<Coupon, Error> {
  let res = query!(
    r#"SELECT
      url,
      user_id,
      minted_at,
      created_at,
      updated_at
    FROM coupon
    WHERE reward_id = $1
      AND number = $2"#,
    reward_id,
    number
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(Coupon {
    reward_id,
    number,
    url: res.url,
    user_id: res.user_id,
    minted_at: res.minted_at,
    created_at: res.created_at,
    updated_at: res.updated_at,
    })
}

// get next number in sequence
pub async fn next_seq(db: &PgPool, reward_id: Uuid) -> Result<i64, Error> {
  match query!(
    r#"SELECT
      MAX(number) as max
    FROM coupon
    WHERE reward_id = $1"#,
    reward_id
  )
  .fetch_one(db)
  .await
  {
    Ok(coupon) => Ok(coupon.max.map(|n| n + 1).unwrap_or(1)),
    Err(err) => Err(handle_pg_error(err)),
  }
}

// create a coupon
pub async fn create_batch(
  db: &PgPool,
  reward_id: Uuid,
  number: i64,
  urls: Vec<String>,
) -> Result<Vec<Coupon>, Error> {
  let mut reward_ids = Vec::<Uuid>::new();
  let mut numbers = Vec::<i64>::new();
  let mut coupons = Vec::<Coupon>::new();
  for (i, url) in urls.iter().enumerate() {
    let number = number + i as i64;
    reward_ids.push(reward_id);
    numbers.push(number);
    coupons.push(Coupon {
      reward_id,
      number,
      url: url.to_owned(),
      user_id: None,
      minted_at: None,
      created_at: Utc::now(),
      updated_at: None,  
    })
  }
  _ = query!(
    "INSERT INTO coupon (
      reward_id,
      number,
      url
    )
    SELECT * FROM UNNEST($1::uuid[], $2::BIGINT[], $3::TEXT[])",
    &reward_ids,
    &numbers,
    &urls,
  )
  .execute(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(coupons)
}

#[derive(Deserialize, IsEmpty)]
pub struct UpdateParams {
  pub url: Option<String>,
  pub user_id: Option<String>,
  pub minted_at: Option<DateTime<Utc>>,
}

// update a coupon
pub async fn update(
  db: &PgPool,
  reward_id: Uuid,
  number: i64,
  p: UpdateParams,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE coupon SET");
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "url" = p.url);
  maybe_bind!(sep, "user_id" = p.user_id);
  maybe_bind!(sep, "minted_at" = p.minted_at);
  query.push(" WHERE reward_id = ").push_bind(reward_id);
  query.push(" AND number = ").push_bind(number);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

#[derive(Deserialize)]
pub struct ReplaceParams {
  pub url: String,
  pub user_id: Option<String>,
  pub minted_at: Option<DateTime<Utc>>,
}

// replace a coupon
pub async fn replace(
  db: &PgPool,
  reward_id: Uuid,
  number: i64,
  p: ReplaceParams,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE coupon SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "url" = p.url);
  must_bind!(sep, "user_id" = p.user_id);
  must_bind!(sep, "minted_at" = p.minted_at);
  query.push(" WHERE reward_id = ").push_bind(reward_id);
  query.push(" AND number = ").push_bind(number);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a coupon
pub async fn delete(db: &PgPool, reward_id: Uuid, number: i64) -> Result<(), Error> {
  match sqlx::query!(
    "DELETE FROM coupon WHERE reward_id = $1 AND number = $2",
    reward_id,
    number
  )
  .execute(db)
  .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}
