use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

use crate::db::sqlx_macro::{maybe_bind, offset_limit};

use super::{
  engage::{Accepted, Messages, Submissions},
  handle_pg_error, maybe_order_by,
  sqlx_macro::must_bind,
  Error, UpdateResult,
};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PubEngage {
  pub id: String,
  pub org_id: Uuid,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub user_id: String,
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

pub async fn get(
  db: &PgPool,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &str,
) -> Result<PubEngage, Error> {
  let res = sqlx::query!(
    r#"SELECT
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
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
    WHERE campaign_id = $1
      AND chain_id = $2
      AND signer_address = $3"#,
    campaign_id,
    chain_id,
    signer_address
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)?;

  Ok(PubEngage {
    id: format!("{}/{}/{}", campaign_id, chain_id, signer_address),
    org_id: res.org_id,
    project_id: res.project_id,
    campaign_id,
    chain_id,
    signer_address: signer_address.to_owned(),
    user_id: res.user_id,
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]

pub struct PubListEngage {
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub user_id: String,
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

#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct PubListFilter {
  pub project_id: Option<Uuid>,
  pub campaign_id: Option<Uuid>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

pub async fn list(
  db: &PgPool,
  chain_id: i64,
  signer_address: &str,
  p: super::ListParams<PubListFilter>,
) -> Result<Vec<PubEngage>, Error> {
  let mut query = QueryBuilder::<Postgres>::new(
    r#"SELECT
      org_id,
      project_id,
      campaign_id,
      chain_id,
      signer_address,
      user_id,
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
  must_bind!(sep, "chain_id" = chain_id);
  must_bind!(sep, "signer_address" = signer_address);
  maybe_bind!(sep, "campaign_id" = p.filter.campaign_id);
  maybe_bind!(sep, "project_id" = p.filter.project_id);
  maybe_order_by(&mut query, &p.order, vec!["created_at"])?;
  offset_limit!(query, p.offset, p.limit);

  let res = query.build().fetch_all(db).await.map_err(handle_pg_error)?;

  let res = res
    .into_iter()
    .map(|en| {
      let campaign_id = en.get("campaign_id");
      PubEngage {
        id: format!("{}/{}/{}", campaign_id, chain_id, signer_address),
        org_id: en.get("org_id"),
        project_id: en.get("project_id"),
        campaign_id,
        chain_id,
        signer_address: signer_address.to_owned(),
        user_id: en.get("user_id"),
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
    .collect::<Vec<PubEngage>>();

  Ok(res)
}

#[derive(Deserialize, IsEmpty, Default)]
pub struct UpdateParam {
  pub country_id: Option<i16>,
}

// update a org
pub async fn update<'a>(
  db: &PgPool,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &'a str,
  p: UpdateParam,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }

  let mut query = QueryBuilder::<Postgres>::new("UPDATE engage SET");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  maybe_bind!(sep, "country_id" = p.country_id);
  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
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

pub async fn submit_proof(
  db: &PgPool,
  campaign_id: Uuid,
  chain_id: i64,
  signer_address: &str,
  p: Submissions,
) -> Result<UpdateResult, Error> {
  if p.is_empty() {
    return Err(Error::EmptyUpdateSet);
  }

  let mut query = QueryBuilder::<Postgres>::new("UPDATE engage SET ");
  let mut sep = query.separated(", ");
  sep.push(" updated_at = NOW() ");
  must_bind!(sep, "submissions = submissions" || Json(&p));
  query.push(" WHERE ");
  let mut sep = query.separated(" AND ");
  for task_id in p.keys() {
    sep
      .push("(NOT accepted ? ")
      .push_bind_unseparated(task_id.clone());
    sep
      .push_unseparated(" OR accepted ->> ")
      .push_bind_unseparated(task_id.to_owned())
      .push_unseparated(" != 'true')");
  }
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
