use std::collections::HashMap;

use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, types::Json, PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::db::{
  campaign::Task,
  sqlx_macro::{maybe_bind, offset_limit},
};

use super::{
  handle_pg_error, maybe_order_by, sqlx_macro::must_bind, CreateResult, Error, UpdateResult,
};

pub type Accepted = HashMap<String, bool>;
pub type Messages = HashMap<String, String>;
pub type Submissions = HashMap<String, Submission>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Engage {
  pub id: String,
  pub org_id: Uuid,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub user_id: String,
  pub user_name: Option<String>,
  pub submissions: Json<Submissions>,
  pub accepted: Json<Accepted>,
  pub messages: Json<Messages>,
  pub coupon_issue_id: Option<String>,
  pub coupon_serial: Option<String>,
  pub coupon_url: Option<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub country_id: Option<i16>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PubEngage {
  pub id: String,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub user_id: String,
  pub user_name: Option<String>,
  pub submissions: Json<Submissions>,
  pub accepted: Json<Accepted>,
  pub messages: Json<Messages>,
  pub coupon_issue_id: Option<String>,
  pub coupon_serial: Option<String>,
  pub coupon_url: Option<String>,
  pub country_id: Option<i16>,
  pub created_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct Submission {
  pub message: Option<String>,
  pub link: Option<String>,
  pub images: Option<Vec<String>>,
}

pub async fn get(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &str,
) -> Result<Engage, Error> {
  let res = sqlx::query!(
    r#"SELECT
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
      user_name,
      submissions AS "submissions: Json<Submissions>",
      accepted AS "accepted: Json<Accepted>",
      messages AS "messages: Json<Messages>",
      coupon_issue_id,
      coupon_serial,
      coupon_url,
      country_id,
      created_at,
      updated_at
    FROM engage
    WHERE org_id = $1
      AND campaign_id = $2
      AND chain_id = $3
      AND signer_address = $4"#,
    org_id,
    campaign_id,
    chain_id,
    signer_address
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(Engage {
    id: format!("{}/{}/{}", campaign_id, chain_id, signer_address),
    org_id: res.org_id,
    project_id: res.project_id,
    campaign_id,
    chain_id,
    signer_address: signer_address.to_owned(),
    user_id: res.user_id,
    user_name: res.user_name,
    submissions: res.submissions,
    accepted: res.accepted,
    messages: res.messages,
    coupon_issue_id: res.coupon_issue_id,
    coupon_serial: res.coupon_serial,
    coupon_url: res.coupon_url,
    country_id: res.country_id,
    created_at: res.created_at,
    updated_at: res.updated_at,
  })
}

pub struct ListParams {
  pub org_id: Uuid,
  pub project_id: Option<Uuid>,
  pub campaign_id: Option<Uuid>,
  pub chain_id: Option<u64>,
  pub signer_address: Option<String>,
  pub user_id: Option<String>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
  pub order: Option<String>,
  pub offset: u64,
  pub limit: u64,
}

pub async fn list(db: &PgPool, p: ListParams) -> Result<Vec<Engage>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
      user_name,
      submissions,
      accepted,
      messages,
      coupon_issue_id,
      coupon_serial,
      coupon_url,
      country_id,
      created_at,
      updated_at
    FROM engage
    WHERE "#,
  );
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = p.org_id);
  maybe_bind!(sep, "project_id" = p.project_id);
  maybe_bind!(sep, "campaign_id" = p.campaign_id);
  maybe_bind!(sep, "chain_id" = p.chain_id.map(|v| v as i64));
  maybe_bind!(sep, "signer_address" = p.signer_address);
  maybe_order_by(&mut query, &p.order, vec!["created_at"])?;
  offset_limit!(query, p.offset, p.limit);

  let res = query.build().fetch_all(db).await.map_err(handle_pg_error)?;

  let res = res
    .into_iter()
    .map(|en| {
      let campaign_id = en.get("campaign_id");
      let chain_id = en.get("chain_id");
      let signer_address: String = en.get("signer_address");
      Engage {
        id: format!("{}/{}/{}", campaign_id, chain_id, signer_address),
        org_id: en.get("org_id"),
        project_id: en.get("project_id"),
        campaign_id,
        chain_id,
        signer_address: signer_address.to_owned(),
        user_id: en.get("user_id"),
        user_name: en.get("user_name"),
        submissions: en.get("submissions"),
        accepted: en.get("accepted"),
        messages: en.get("messages"),
        coupon_issue_id: en.get("coupon_issue_id"),
        coupon_serial: en.get("coupon_serial"),
        coupon_url: en.get("coupon_url"),
        country_id: en.get("country_id"),
        created_at: en.get("created_at"),
        updated_at: en.get("updated_at"),
      }
    })
    .collect::<Vec<Engage>>();

  Ok(res)
}

#[derive(Deserialize, Serialize)]
pub struct CreateParam<'a> {
  pub org_id: Uuid,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: &'a str,
  pub user_id: &'a str,
  pub user_name: Option<String>,
  pub submissions: Submissions,
}

// create an engagement
pub async fn create<'a>(db: &PgPool, p: CreateParam<'a>) -> Result<CreateResult, Error> {
  let res = sqlx::query_as!(
    CreateResult,
    "INSERT INTO engage (
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
      user_name,
      submissions)
    VALUES (
      $1,
      $2,
      $3,
      $4,
      $5,
      $6,
      $7,
      $8)
    RETURNING created_at",
    p.org_id,
    p.project_id,
    p.campaign_id,
    p.chain_id,
    p.signer_address,
    p.user_id,
    p.user_name,
    Json(&p.submissions) as _,
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  _ = super::project_membership::join(db, p.project_id, p.user_id)
  .await?;

  Ok(res)
}

#[derive(Deserialize, IsEmpty, Default)]
pub struct UpdateParam<'a> {
  pub coupon_issue_id: Option<&'a str>,
  pub coupon_serial: Option<&'a str>,
  pub coupon_url: Option<&'a str>,
  pub country_id: Option<i16>,
}

// update an engagement
pub async fn update<'a>(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &str,
  p: UpdateParam<'a>,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }

  let mut query = QueryBuilder::<Postgres>::new("UPDATE engage SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "coupon_issue_id" = p.coupon_issue_id);
  maybe_bind!(sep, "coupon_serial" = p.coupon_serial);
  maybe_bind!(sep, "coupon_url" = p.coupon_url);
  maybe_bind!(sep, "country_id" = p.country_id);
  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = org_id);
  must_bind!(sep, "campaign_id" = campaign_id);
  must_bind!(sep, "chain_id" = chain_id);
  must_bind!(sep, "signer_address" = signer_address);
  query.push(" RETURNING updated_at");

  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

#[derive(Deserialize)]
pub struct ReplaceParams {
  pub coupon_issue_id: Option<String>,
  pub coupon_serial: Option<String>,
  pub coupon_url: Option<String>,
  pub country_id: Option<i16>,
}

// delete an engagement
pub async fn delete(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: String,
) -> Result<(), Error> {
  match sqlx::query!(
    "DELETE FROM engage WHERE org_id = $1 AND campaign_id = $2 AND chain_id = $3 AND signer_address = $4",
    org_id,
    campaign_id,
    chain_id,
    signer_address
  )
  .execute(db)
  .await
  {
    Ok(_) => Ok(()),
    Err(err) => Err(handle_pg_error(err)),
  }
}

pub async fn approve(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: String,
  accepted: Accepted,
) -> Result<UpdateResult, Error> {
  if accepted.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut tx = db.begin().await?;

  let campaign = query!(
    r#"SELECT
      project_id,
      tasks AS "tasks: Json<Vec<Task>>",
      voucher_policy,
      voucher_expire_at,
      end_at
    FROM campaign
    WHERE id = $1"#,
    campaign_id
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let engage = query!(
    r#"SELECT
      user_id,
      accepted AS "accepted: Json<Accepted>"
    FROM engage
    WHERE campaign_id = $1
      AND chain_id = $2
      AND signer_address = $3"#,
    campaign_id,
    chain_id,
    signer_address
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error)?;

  let num_tasks = campaign.tasks.0.len();
  for task in campaign.tasks.0 {
    if let Some(amount) = task.point {
      if let Some(accept) = accepted.get(&task.id) {
        let old_accept = engage.accepted.0.get(&task.id).unwrap_or(&false);
        if *accept != *old_accept {
          if *accept {
            let valid_from = match campaign.voucher_policy {
              1 => Some(Utc::now().date_naive()),
              2 => {
                if engage.accepted.len() == num_tasks {
                  query!(
                    "UPDATE voucher
                    SET valid_from = NOW()
                    WHERE org_id = $1
                      AND campaign_id = $2
                      AND chain_id = $3
                      AND signer_address = $4",
                    org_id,
                    campaign_id,
                    chain_id,
                    signer_address,
                  )
                  .execute(&mut *tx)
                  .await
                  .map_err(handle_pg_error)?;
                } else {
                  query!(
                    "UPDATE voucher
                    SET valid_from = NULL
                    WHERE org_id = $1
                      AND campaign_id = $2
                      AND chain_id = $3
                      AND signer_address = $4",
                    org_id,
                    campaign_id,
                    chain_id,
                    signer_address,
                  )
                  .execute(&mut *tx)
                  .await
                  .map_err(handle_pg_error)?;
                }
                Some(Utc::now().date_naive())
              }
              3 => campaign.end_at,
              _ => None,
              // TODO: Handle reward on specific date
            };
            /*
            -- 0: Reward manually
            -- 1: Reward on task completion
            -- 2: Reward on all task completion
            -- 3: Reward at campain end
            -- 4: Reward on a specific date
            */
            query!(
              "INSERT INTO voucher (
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
                created_at
              ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                $10,
                $11,
                NOW()
              )",
              org_id,
              campaign.project_id,
              campaign_id,
              chain_id,
              signer_address,
              engage.user_id,
              task.id,
              amount,
              amount,
              valid_from,
              campaign.voucher_expire_at
            )
            .execute(&mut *tx)
            .await
            .map_err(handle_pg_error)?;
          } else {
            query!(
              "DELETE
              FROM voucher 
              WHERE campaign_id = $1
                AND chain_id = $2
                AND signer_address = $3
                AND task_id = $4",
              campaign_id,
              chain_id,
              signer_address,
              task.id,
            )
            .execute(&mut *tx)
            .await
            .map_err(handle_pg_error)?;
          }
        }
      }
    }
  }

  let up_res = query_as!(
    UpdateResult,
    r#"UPDATE engage
    SET
      updated_at = NOW(),
      accepted = accepted || $1
    WHERE org_id = $2
      AND campaign_id = $3
      AND chain_id = $4
      AND signer_address = $5
    RETURNING
      updated_at AS "updated_at!""#,
    Json(&accepted) as _,
    org_id,
    campaign_id,
    chain_id,
    signer_address
  )
  .fetch_one(&mut *tx)
  .await
  .map_err(handle_pg_error);

  tx.commit().await?;

  up_res
}

#[derive(Deserialize, IsEmpty)]
pub struct UpdateCouponSet {
  coupon_url: Option<String>,
  coupon_serial: Option<String>,
  coupon_issue_id: Option<String>,
}

pub async fn update_coupon(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: String,
  p: UpdateCouponSet,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }
  let mut query = QueryBuilder::<Postgres>::new("UPDATE engage SET ");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "coupon_issue_id" = p.coupon_issue_id);
  maybe_bind!(sep, "coupon_serial" = p.coupon_serial);
  maybe_bind!(sep, "coupon_url" = p.coupon_url);
  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = org_id);
  must_bind!(sep, "campaign_id" = campaign_id);
  must_bind!(sep, "chain_id" = chain_id);
  must_bind!(sep, "signer_address" = signer_address);
  query.push(" RETURNING updated_at");

  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}

pub async fn delete_coupon_url<'a>(
  db: &PgPool,
  org_id: Uuid,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &'a str,
) -> Result<UpdateResult, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    "UPDATE engage
    SET
      coupon_issue_id = NULL,
      coupon_serial = NULL,
      coupon_url = NULL,
      updated_at = NOW()
    WHERE ",
  );
  let mut sep = query.separated(" AND ");
  must_bind!(sep, "org_id" = org_id);
  must_bind!(sep, "campaign_id" = campaign_id);
  must_bind!(sep, "chain_id" = chain_id);
  must_bind!(sep, "signer_address" = signer_address);
  query.push(" RETURNING updated_at");

  query
    .build_query_as()
    .fetch_one(db)
    .await
    .map_err(handle_pg_error)
}
