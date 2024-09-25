use chrono::{DateTime, NaiveDate, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use sqlx::{prelude::FromRow, query, query_as, PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::db::sqlx_macro::{must_bind, maybe_bind, offset_limit};

use super::{coupon::Coupon, handle_pg_error, maybe_order_by, CreateResult, Error, UpdateResult};

#[derive(FromRow, Serialize)]
pub struct Reward {
  pub id: Uuid,
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub description: String,
  pub tandc: Option<String>,
  pub images: Vec<String>,
  pub active_from: Option<NaiveDate>,
  pub active_until: Option<NaiveDate>,
  pub valid_from: Option<NaiveDate>,
  pub valid_until: Option<NaiveDate>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct RewardFilter {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

// list rewards
pub async fn list(db: &PgPool, p: super::ListParams<RewardFilter>) -> Result<Vec<Reward>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      id,
      issuer_id,
      category,
      country_id,
      name,
      description,
      tandc,
      images,
      active_from,
      active_until,
      valid_from,
      valid_until,
      created_at,
      updated_at
    FROM reward"#,
  );

  if !p.filter.is_empty() {
    query.push(" WHERE ");
    let mut sep = query.separated(" AND ");
    maybe_bind!(sep, "issuer_id" = p.filter.issuer_id);
    maybe_bind!(sep, "category" = p.filter.category);
    maybe_bind!(sep, "country_id" = p.filter.country_id);
    maybe_bind!(sep, "created_after" <= p.filter.created_after);
    maybe_bind!(sep, "created_before" >= p.filter.created_before);
  }

  maybe_order_by(
    &mut query,
    &p.order,
    vec![
      "active_from",
      "active_until",
      "valid_from",
      "valid_until",
      "created_at",
      "updated_at",
    ],
  )?;
  offset_limit!(query, p.offset, p.limit);

  query
    .build_query_as()
    .fetch_all(db)
    .await
    .map_err(handle_pg_error)
}

// get a reward
pub async fn get(db: &PgPool, id: Uuid) -> Result<Reward, Error> {
  query_as!(
    Reward,
    r#"SELECT
      id,
      issuer_id,
      category,
      country_id,
      name,
      description,
      tandc,
      images,
      active_from,
      active_until,
      valid_from,
      valid_until,
      created_at,
      updated_at
    FROM reward
    WHERE id = $1"#,
    id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

pub struct CreateParam {
  pub id: Uuid,
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub description: Option<String>,
  pub tandc: Option<String>,
  pub images: Vec<String>,
  pub active_from: Option<NaiveDate>,
  pub active_until: Option<NaiveDate>,
  pub valid_from: Option<NaiveDate>,
  pub valid_until: Option<NaiveDate>,
}

// create a reward
pub async fn create(db: &PgPool, p: CreateParam) -> Result<CreateResult, Error> {
  query_as!(
    CreateResult,
    "INSERT INTO reward (
      id,
      issuer_id,
      category,
      country_id,
      name,
      description,
      tandc,
      images,
      active_from,
      active_until,
      valid_from,
      valid_until
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    RETURNING created_at",
    p.id,
    p.issuer_id,
    p.category,
    p.country_id,
    p.name,
    p.description,
    p.tandc,
    &p.images,
    p.active_from,
    p.active_until,
    p.valid_from,
    p.valid_until,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}

#[serde_as]
#[derive(Deserialize, IsEmpty)]
pub struct UpdateParams {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub tandc: Option<String>,
  pub images: Option<Vec<String>>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_until: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_until: Option<NaiveDate>,
}

// update a reward
pub async fn update(db: &PgPool, id: Uuid, p: UpdateParams) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE reward SET");
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "issuer_id" = p.issuer_id);
  maybe_bind!(sep, "category" = p.category);
  maybe_bind!(sep, "country_id" = p.country_id);
  maybe_bind!(sep, "name" = p.name);
  maybe_bind!(sep, "description" = p.description);
  maybe_bind!(sep, "tandc" = p.tandc);
  maybe_bind!(sep, "images" = p.images);
  maybe_bind!(sep, "active_from" = p.active_from);
  maybe_bind!(sep, "active_until" = p.active_until);
  maybe_bind!(sep, "valid_from" = p.valid_from);
  maybe_bind!(sep, "valid_until" = p.valid_until);
  query.push(" WHERE id = ").push_bind(id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

#[serde_as]
#[derive(Deserialize)]
pub struct ReplaceParams {
  pub issuer_id: Option<String>,
  pub category: Option<i16>,
  pub country_id: Option<i16>,
  pub name: String,
  pub description: Option<String>,
  pub tandc: Option<String>,
  pub images: Vec<String>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub active_until: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_from: Option<NaiveDate>,
  #[serde_as(as = "NoneAsEmptyString")]
  pub valid_until: Option<NaiveDate>,
}

// replace a reward
pub async fn replace(db: &PgPool, id: Uuid, p: ReplaceParams) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new("UPDATE reward SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "issuer_id" = p.issuer_id);
  must_bind!(sep, "category" = p.category);
  must_bind!(sep, "country_id" = p.country_id);
  must_bind!(sep, "name" = p.name);
  must_bind!(sep, "description" = p.description);
  must_bind!(sep, "tandc" = p.tandc);
  must_bind!(sep, "images" = p.images);
  must_bind!(sep, "active_from" = p.active_from);
  must_bind!(sep, "active_until" = p.active_until);
  must_bind!(sep, "valid_from" = p.valid_from);
  must_bind!(sep, "valid_until" = p.valid_until);
  query.push(" WHERE id = ").push_bind(id);
  query.push(" RETURNING updated_at");
  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

// delete a reward
pub async fn delete(db: &PgPool, id: Uuid) -> Result<(), Error> {
  match sqlx::query!("DELETE FROM reward WHERE id = $1", id,)
    .execute(db)
    .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}

#[derive(sqlx::FromRow, Serialize, Debug)]
pub struct PointRewardUpdateResult {
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub task_id: String,
  pub minted: i64,
  pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct MintProjectRewardResult {
  pub coupon: Coupon,
  pub vouchers: Vec<PointRewardUpdateResult>,
  pub balance: i64,
}

// replace a reward
pub async fn mint_project_reward(
  db: &PgPool,
  user_id: &str,
  reward_id: Uuid,
  project_id: Uuid,
) -> Result<MintProjectRewardResult, Error> {
  let mut tx = db.begin().await?;

  let coupon = query!(
    r#"SELECT
    reward_id,
    number,
    url,
    created_at
  FROM coupon
  WHERE user_id IS NULL
    AND minted_at IS NULL
    AND reward_id = $1
  ORDER BY number ASC
  LIMIT 1
  "#,
    reward_id
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let project_reward = query!(
    r#"SELECT
      point,
      max_mint,
      user_mint
    FROM project__reward
    WHERE project_id = $1
      AND reward_id = $2
      AND active = true
      AND approved = true
      AND point IS NOT NULL
      AND (max_mint IS NULL OR (SELECT COUNT(user_id) FROM voucher WHERE project_id = $1) < max_mint)
      "#,
    project_id,
    reward_id
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  if let Some(user_mint) = project_reward.user_mint {
    let c = query!(
      r#"SELECT
        COUNT(user_id) AS "count!"
      FROM coupon
      WHERE user_id = $1
        AND reward_id = $2
        AND minted_at IS NULL
      LIMIT 1
      "#,
      user_id,
      reward_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_pg_error)?;
    if c.count >= user_mint {
      return Err(Error::LimitReached);
    }
  }

  let point = query!(
    r#"SELECT
      SUM(balance)::BIGINT
    FROM voucher
    WHERE project_id = $1
      AND user_id = $2
      AND valid_from <= NOW()
      AND (valid_until IS NULL OR valid_until >= NOW())
      "#,
    project_id,
    user_id,
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let point_total = point.sum.unwrap_or_default();

  if point_total >= project_reward.point {
    let vouchers = query!(
      r#"SELECT
      campaign_id,
      chain_id,
      signer_address,
      task_id,
      balance
    FROM voucher
    WHERE project_id = $1
      AND user_id = $2
      AND valid_from <= NOW()
      AND (valid_until IS NULL OR valid_until >= NOW())
    ORDER BY
      valid_until ASC,
      created_at ASC
      "#,
      project_id,
      user_id,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(handle_pg_error)?;

    let mut vouchers_update_results = Vec::<PointRewardUpdateResult>::new();
    let mut total: i64 = 0;
    for r in vouchers {
      total += r.balance;
      if total < project_reward.point {
        let res = query_as!(
          super::UpdateResult,
          r#"UPDATE voucher SET
            balance = 0,
            updated_at = NOW()
          WHERE campaign_id = $1
            AND chain_id  = $2
            AND signer_address  = $3
            AND task_id  = $4
          RETURNING updated_at AS "updated_at!"
          "#,
          r.campaign_id,
          r.chain_id,
          r.signer_address,
          r.task_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(handle_pg_error)?;
        vouchers_update_results.push(PointRewardUpdateResult {
          campaign_id: r.campaign_id,
          chain_id: r.chain_id,
          signer_address: r.signer_address,
          task_id: r.task_id,
          minted: r.balance,
          updated_at: res.updated_at,
        });
      } else {
        let remainder = total - project_reward.point;
        let res = query_as!(
          super::UpdateResult,
          r#"UPDATE voucher SET
            balance = $5,
            updated_at = NOW()
          WHERE campaign_id = $1
            AND chain_id  = $2
            AND signer_address  = $3
            AND task_id  = $4
          RETURNING updated_at AS "updated_at!"
          "#,
          r.campaign_id,
          r.chain_id,
          r.signer_address,
          r.task_id,
          remainder
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(handle_pg_error)?;
        vouchers_update_results.push(PointRewardUpdateResult {
          campaign_id: r.campaign_id,
          chain_id: r.chain_id,
          signer_address: r.signer_address,
          task_id: r.task_id,
          minted: r.balance,
          updated_at: res.updated_at,
        });
        break;
      }
    }

    let res = query!(
      r#"UPDATE coupon SET
        user_id = $3,
        minted_at = NOW(),
        updated_at = NOW()
      WHERE user_id IS NULL
        AND minted_at IS NULL
        AND reward_id = $1
        AND number = $2
      RETURNING minted_at AS "minted_at!"
      "#,
      reward_id,
      coupon.number,
      user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_pg_error)?;

    tx.commit().await.map_err(handle_pg_error)?;

    Ok(MintProjectRewardResult {
      coupon: Coupon {
        reward_id,
        number: coupon.number,
        url: coupon.url,
        user_id: Some(user_id.to_owned()),
        minted_at: Some(res.minted_at),
        created_at: coupon.created_at,
        updated_at: Some(res.minted_at),
      },
      vouchers: vouchers_update_results,
      balance: point_total - project_reward.point,
    })
  } else {
    Err(Error::NotFound)
  }
}

#[derive(sqlx::FromRow, Serialize)]
pub struct MintCampaignRewardResult {
  pub coupon: Coupon,
  pub vouchers: Vec<PointRewardUpdateResult>,
  pub balance: i64,
}

// replace a reward
pub async fn mint_campaign_reward(
  db: &PgPool,
  user_id: &str,
  reward_id: Uuid,
  campaign_id: Uuid,
) -> Result<MintCampaignRewardResult, Error> {
  let mut tx = db.begin().await?;

  let coupon = query!(
    r#"SELECT
    reward_id,
    number,
    url,
    created_at
  FROM coupon
  WHERE user_id IS NULL
    AND minted_at IS NULL
    AND reward_id = $1
  ORDER BY number ASC
  LIMIT 1
  "#,
    reward_id
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let campaign_reward = query!(
    r#"SELECT
      point,
      max_mint,
      user_mint
    FROM campaign__reward
    WHERE campaign_id = $1
      AND reward_id = $2
      AND active = true
      AND approved = true
      AND point IS NOT NULL
      AND (max_mint IS NULL
        OR max_mint = 0
        OR (
          SELECT COUNT(user_id)
          FROM coupon
          WHERE reward_id = $2
        ) < max_mint)
      "#,
    campaign_id,
    reward_id
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let user_mint = campaign_reward.user_mint.unwrap_or_default();
  if user_mint > 0 {
    let c = query!(
      r#"SELECT
        COUNT(user_id) AS "count!"
      FROM coupon
      WHERE user_id = $1
        AND reward_id = $2
        AND minted_at IS NULL
      LIMIT 1
      "#,
      user_id,
      reward_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_pg_error)?;
    if c.count >= user_mint {
      return Err(Error::LimitReached);
    }
  }

  let point = query!(
    r#"SELECT
      SUM(balance)::BIGINT
    FROM voucher
    WHERE campaign_id = $1
      AND user_id = $2
      AND valid_from <= NOW()
      AND (valid_until IS NULL OR valid_until >= NOW())
      "#,
    campaign_id,
    user_id,
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let point_total = point.sum.unwrap_or_default();

  if point_total >= campaign_reward.point {
    let vouchers = query!(
      r#"SELECT
      campaign_id,
      chain_id,
      signer_address,
      task_id,
      balance
    FROM voucher
    WHERE campaign_id = $1
      AND user_id = $2
      AND valid_from <= NOW()
      AND (valid_until IS NULL OR valid_until >= NOW())
    ORDER BY
      valid_until ASC,
      created_at ASC
      "#,
      campaign_id,
      user_id,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(handle_pg_error)?;

    let mut vouchers_update_results = Vec::<PointRewardUpdateResult>::new();
    let mut total: i64 = 0;
    for r in vouchers {
      total += r.balance;
      if total < campaign_reward.point {
        let res = query_as!(
          super::UpdateResult,
          r#"UPDATE voucher SET
            balance = 0,
            updated_at = NOW()
          WHERE campaign_id = $1
            AND chain_id  = $2
            AND signer_address  = $3
            AND task_id  = $4
          RETURNING updated_at AS "updated_at!"
          "#,
          r.campaign_id,
          r.chain_id,
          r.signer_address,
          r.task_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(handle_pg_error)?;
        vouchers_update_results.push(PointRewardUpdateResult {
          campaign_id: r.campaign_id,
          chain_id: r.chain_id,
          signer_address: r.signer_address,
          task_id: r.task_id,
          minted: r.balance,
          updated_at: res.updated_at,
        });
      } else {
        let remainder = total - campaign_reward.point;
        let res = query_as!(
          super::UpdateResult,
          r#"UPDATE voucher SET
            balance = $5,
            updated_at = NOW()
          WHERE campaign_id = $1
            AND chain_id  = $2
            AND signer_address  = $3
            AND task_id  = $4
          RETURNING updated_at AS "updated_at!"
          "#,
          r.campaign_id,
          r.chain_id,
          r.signer_address,
          r.task_id,
          remainder
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(handle_pg_error)?;
        vouchers_update_results.push(PointRewardUpdateResult {
          campaign_id: r.campaign_id,
          chain_id: r.chain_id,
          signer_address: r.signer_address,
          task_id: r.task_id,
          minted: r.balance,
          updated_at: res.updated_at,
        });
        break;
      }
    }

    let res = query!(
      r#"UPDATE coupon SET
        user_id = $3,
        minted_at = NOW(),
        updated_at = NOW()
      WHERE user_id IS NULL
        AND minted_at IS NULL
        AND reward_id = $1
        AND number = $2
      RETURNING minted_at AS "minted_at!"
      "#,
      reward_id,
      coupon.number,
      user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_pg_error)?;

    tx.commit().await.map_err(handle_pg_error)?;

    Ok(MintCampaignRewardResult {
      coupon: Coupon {
        reward_id,
        number: coupon.number,
        url: coupon.url,
        user_id: Some(user_id.to_owned()),
        minted_at: Some(res.minted_at),
        created_at: coupon.created_at,
        updated_at: Some(res.minted_at),
      },
      vouchers: vouchers_update_results,
      balance: point_total - campaign_reward.point,
    })
  } else {
    Err(Error::NotFound)
  }
}
