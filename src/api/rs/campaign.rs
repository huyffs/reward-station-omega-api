use axum::{
  extract::{Path, State},
  response::Response,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use is_empty::IsEmpty;
use serde::Deserialize;
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};
use uuid::Uuid;

use crate::{
  api::handle_result,
  db::{self, campaign::CampaignFilter},
};

pub async fn get(State(db): State<sqlx::PgPool>, Path(campaign_id): Path<Uuid>) -> Response {
  let res = db::campaign_pub::get(&db, campaign_id);

  handle_result(res.await)
}

pub async fn get_org_id(State(db): State<sqlx::PgPool>, Path(campaign_id): Path<Uuid>) -> Response {
  let res = db::campaign_pub::get_org_id(&db, campaign_id);

  handle_result(res.await)
}

#[serde_as]
#[derive(IsEmpty, Deserialize, Clone, Debug)]
pub struct ListFilter {
  pub chain_id: Option<u64>,
  pub project_id: Option<Uuid>,
  #[serde_as(as = "Option<StringWithSeparator::<CommaSeparator, String>>")]
  pub contracts: Option<Vec<String>>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  Query(p): Query<db::ListParams<ListFilter>>,
) -> Response {
  let res = db::campaign_pub::list(
    &db,
    db::ListParams::<CampaignFilter> {
      filter: CampaignFilter {
        org_id: None,
        project_id: p.filter.project_id,
        chain_id: p.filter.chain_id,
        contracts: p.filter.contracts,
        created_after: p.filter.created_after,
        created_before: p.filter.created_before,
      },
      order: p.order,
      offset: p.offset,
      limit: p.limit,
    },
  );

  handle_result(res.await)
}
