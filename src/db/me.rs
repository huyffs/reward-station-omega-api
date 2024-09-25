use serde::Serialize;
use sqlx::{prelude::FromRow, query, PgPool};

use super::{handle_pg_error, Error};

#[derive(FromRow, Serialize, Debug)]
pub struct Me {
  pub xp: i64,
  pub club: i64,
}

// get a user's xp
pub async fn get_xp(db: &PgPool, user_id: &str) -> Result<i64, Error> {
  let res = query!(
    r#"SELECT
      COUNT(user_id)
    FROM coupon
    WHERE user_id = $1"#,
    user_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(res.count.unwrap_or_default())
}

// get a user's club memberships
pub async fn get_club(db: &PgPool, user_id: &str) -> Result<i64, Error> {
  let res = query!(
    r#"SELECT
      COUNT(user_id)
    FROM project__user
    WHERE user_id = $1"#,
    user_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(res.count.unwrap_or_default())
}
