use axum::{
  extract::{Path, Query, State},
  http::StatusCode,
  response::{sse::Event, IntoResponse, Json, Response, Sse},
};
use chrono::{DateTime, Utc};
use futures_util::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use sqlx::postgres::PgListener;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast::Sender;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::{
  api::{handle_db_error, handle_result, into_json_response},
  auth::MyFirebaseUser,
  db::{self, engage::Submissions, engage_event::EngageEventLog, Never},
};

#[derive(Error, Debug)]
pub enum EngageEventError {
  #[error("unable to determine event kind")]
  Unknown,
}

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((chain_id, signer_address, campaign_id)): Path<(i64, String, Uuid)>,
) -> Response {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }
  let res = db::engage_pub::get(&db, campaign_id, chain_id, signer_address);

  handle_result(res.await)
}

pub async fn get_tasks(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<db::ListParams<db::engage_pub::PubListFilter>>,
  Path((chain_id, signer_address)): Path<(i64, String)>,
) -> Response {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage_pub::list(&db, chain_id, signer_address, p);

  handle_result(res.await)
}

#[derive(Serialize, Debug)]
pub struct RetrieveCouponResponse {
  coupon_url: String,
}
/*
pub async fn retrieve_coupon<'a>(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  State(mut mezzofy_client): State<mezzofy::Client>,
  Path((chain_id, signer_address, campaign_id)): Path<(i64, String, Uuid)>,
) -> Result<Response, Response> {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    Err(StatusCode::FORBIDDEN.into_response())?
  }

  let en = db::engage_pub::get(&db, campaign_id, chain_id, signer_address)
    .await
    .map_err(handle_db_error)?;

  let coupon_url = match en.coupon_url {
    Some(url) => Ok(url),
    None => match en.coupon_serial {
      Some(coupon_serial) => {
        let res = mezzofy_client
          .get_coupon(&coupon_serial)
          .await
          .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()).into_response())?;

        let url = res.couponserial.serial.single_serial_url;

        _ = db::engage::update(
          &db,
          en.org_id,
          campaign_id,
          chain_id,
          signer_address,
          UpdateParam {
            coupon_issue_id: None,
            coupon_serial: None,
            coupon_url: Some(&url),
            country_id: None,
          },
        )
        .await
        .map_err(handle_db_error);

        Ok(url.to_owned())
      }
      None => Err(
        (
          StatusCode::PRECONDITION_FAILED,
          String::from("coupon not yet issued"),
        )
          .into_response(),
      ),
    },
  }?;

  Ok(into_json_response(&RetrieveCouponResponse { coupon_url }))
}
 */
#[derive(Deserialize, Debug)]
pub struct CreateEngageForm {
  pub submissions: db::engage::Submissions,
}

#[derive(Serialize)]
pub struct Created {
  created_at: DateTime<Utc>,
}

pub async fn create(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((chain_id, signer_address, campaign_id)): Path<(i64, String, Uuid)>,
  Json(submissions): Json<Submissions>,
) -> Result<Response, Response> {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    Err(StatusCode::FORBIDDEN.into_response())?;
  }
  let campaign = db::campaign_pub::get_relation_ids(&db, campaign_id)
    .await
    .map_err(handle_db_error)?;

  let res = db::engage::create(
    &db,
    db::engage::CreateParam {
      org_id: campaign.org_id,
      project_id: campaign.project_id,
      campaign_id,
      chain_id,
      signer_address,
      submissions,
      user_id: &user.sub,
      user_name: user.name,
    },
  )
  .await
  .map_err(handle_db_error)?;

  Ok(into_json_response(&res))
}

#[derive(Deserialize, Debug)]
pub struct UpdateEngageForm {
  pub country_id: u16,
}

pub async fn update(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((chain_id, signer_address, campaign_id)): Path<(i64, String, Uuid)>,
  Json(form): Json<UpdateEngageForm>,
) -> Response {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage_pub::update(
    &db,
    campaign_id,
    chain_id,
    signer_address,
    db::engage_pub::UpdateParam {
      country_id: Some(form.country_id as i16),
    },
  );

  handle_result(res.await)
}

pub async fn submit_proof(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((chain_id, signer_address, campaign_id)): Path<(i64, String, Uuid)>,
  Json(form): Json<Submissions>,
) -> Response {
  let signer_address = &signer_address.to_lowercase();
  if !user.has_wallet_claim(chain_id, signer_address) {
    return StatusCode::FORBIDDEN.into_response();
  }
  if form.values().any(|sub| {
    sub.link.is_none()
      && sub.message.is_none()
      && (sub.images.is_none() || sub.images.as_ref().unwrap().is_empty())
  }) {
    return StatusCode::BAD_REQUEST.into_response();
  }

  let res = db::engage_pub::submit_proof(&db, campaign_id, chain_id, signer_address, form);

  handle_result(res.await)
}

#[derive(Clone, Serialize_repr, Debug)]
#[repr(u8)]
pub enum EngageKind {
  SubmittedProof = 1,
  SubmissionApproved = 2,
  Issued = 3,
  Claimed = 4,
}

#[derive(Clone, Serialize, Debug)]
pub struct EngageEvent {
  pub id: i64,
  pub project_id: Uuid,
  pub campaign_id: Uuid,
  pub chain_id: i64,
  pub signer_address: String,
  pub kind: EngageKind,
  pub task_ids: Option<Vec<String>>,
}

impl TryFrom<EngageEventLog> for EngageEvent {
  type Error = EngageEventError;

  fn try_from(p: EngageEventLog) -> Result<Self, Self::Error> {
    if p.old_coupon_url != p.new_coupon_url {
      Ok(Self::make(p, EngageKind::Claimed, None))
    } else if p.old_coupon_serial != p.new_coupon_serial {
      Ok(Self::make(p, EngageKind::Issued, None))
    } else if let Some(task_ids) = get_accepted_task_ids(&p) {
      Ok(Self::make(
        p,
        EngageKind::SubmissionApproved,
        Some(task_ids),
      ))
    } else if let Some(task_ids) = get_submission_task_ids(&p) {
      Ok(Self::make(p, EngageKind::SubmittedProof, Some(task_ids)))
    } else {
      Err(EngageEventError::Unknown)
    }
  }
}

impl EngageEvent {
  fn make(p: EngageEventLog, kind: EngageKind, task_ids: Option<Vec<String>>) -> Self {
    Self {
      id: p.id,
      project_id: p.project_id,
      campaign_id: p.campaign_id,
      chain_id: p.chain_id,
      signer_address: p.signer_address,
      kind,
      task_ids,
    }
  }
}

pub type EngageEventStream = Sender<EngageEvent>;

pub async fn events(
  State(engages_stream): State<EngageEventStream>,
) -> Sse<impl Stream<Item = Result<Event, anyhow::Error>>> {
  let rx = engages_stream.subscribe();

  let receiver = BroadcastStream::new(rx);
  let stream = receiver.map(|message| {
    let message = message?;
    let data = serde_json::to_string(&message)?;
    Ok(Event::default().data(data))
  });

  Sse::new(stream).keep_alive(
    axum::response::sse::KeepAlive::new()
      .interval(Duration::from_secs(1))
      .text("You good bruv!"),
  )
}

pub async fn list_events(
  State(db): State<sqlx::PgPool>,
  Query(p): Query<db::ListParams<Never>>,
) -> Result<Response, Response> {
  let logs = db::engage_event::list(&db, p)
    .await
    .map_err(handle_db_error)?;

  let mut events = Vec::<EngageEvent>::new();
  for log in logs {
    let l = EngageEventLog {
      id: log.id,
      org_id: log.org_id,
      project_id: log.project_id,
      campaign_id: log.campaign_id,
      chain_id: log.chain_id,
      signer_address: log.signer_address,
      user_id: log.user_id,
      old_submissions: log.old_submissions,
      old_accepted: log.old_accepted,
      old_coupon_serial: log.old_coupon_serial,
      old_coupon_url: log.old_coupon_url,
      new_submissions: log.new_submissions,
      new_accepted: log.new_accepted,
      new_coupon_serial: log.new_coupon_serial,
      new_coupon_url: log.new_coupon_url,
      created_at: log.created_at,
    };
    if let Ok(ev) = EngageEvent::try_from(l) {
      events.push(ev)
    }
  }

  Ok(into_json_response(&events))
}

pub async fn start_listening(
  mut listener: PgListener,
  tx: &EngageEventStream,
) -> Result<(), anyhow::Error> {
  listener.listen("engages").await?;
  loop {
    if let Some(notif) = listener.try_recv().await? {
      match serde_json::from_str::<EngageEventLog>(notif.payload()) {
        Err(e) => {
          tracing::error!("Error deserialize message: {}", e.to_string());
        }
        Ok(payload) => match EngageEvent::try_from(payload) {
          Err(e) => {
            tracing::warn!("Failed to convert payload into event: {}", e.to_string());
          }
          Ok(ev) => match tx.send(ev) {
            Err(e) => {
              tracing::error!("Error send message to client: {}", e.to_string());
            }
            Ok(n) => {
              tracing::info!("Sent event to {} subscribers", n);
            }
          },
        },
      }
    }
  }
}

fn get_accepted_task_ids(p: &EngageEventLog) -> Option<Vec<String>> {
  let mut task_ids = Vec::<String>::new();
  if let Some(new_accepted) = p.new_accepted.as_ref() {
    let it = new_accepted.0.iter();
    match &p.old_accepted {
      Some(old) => {
        for (k, v) in it {
          if *v && !old.contains_key(k) {
            task_ids.push(k.to_string());
          }
        }
      }
      None => {
        for (k, v) in it {
          if *v {
            task_ids.push(k.to_string())
          }
        }
      }
    }
  }
  if task_ids.is_empty() {
    None
  } else {
    Some(task_ids)
  }
}

fn get_submission_task_ids(p: &EngageEventLog) -> Option<Vec<String>> {
  let mut task_ids = Vec::<String>::new();
  if let Some(new_subs) = p.new_submissions.as_ref() {
    let keys = new_subs.keys();
    match &p.old_submissions {
      Some(old_subs) => {
        for k in keys {
          if old_subs.get(k).is_none() {
            task_ids.push(k.to_owned());
          }
        }
      }
      None => {
        for k in keys {
          task_ids.push(k.to_owned());
        }
      }
    }
  }
  if task_ids.is_empty() {
    None
  } else {
    Some(task_ids)
  }
}
