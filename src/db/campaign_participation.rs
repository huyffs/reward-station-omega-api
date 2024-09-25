use serde::Serialize;
use serde_with::skip_serializing_none;
use sqlx::{prelude::FromRow, query_as, PgPool};
use uuid::Uuid;

use crate::db::handle_pg_error;

#[skip_serializing_none]
#[derive(Serialize, FromRow)]
pub struct CampaignParticipation {
  pub campaign_id: Uuid,
  pub user_id: String,
  pub balance: i64,
  pub point: i64,
}

pub async fn list(
  db: &PgPool,
  user_id: &str,
  campaign_id: Uuid,
) -> Result<CampaignParticipation, super::Error> {
  query_as!(
    CampaignParticipation,
    r#"SELECT
      campaign_id,
      user_id,
      SUM(balance)::BIGINT AS "balance!",
      SUM(value)::BIGINT AS "point!"
    FROM voucher
    WHERE user_id = $1
      AND campaign_id = $2
    GROUP BY (campaign_id, user_id)
    "#,
    user_id,
    campaign_id
  )
  .fetch_one(db)
  .await
  .map_err(handle_pg_error)
}
