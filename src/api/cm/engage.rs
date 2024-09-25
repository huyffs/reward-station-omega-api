use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use iso3166::Country;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use uuid::Uuid;

use crate::{
  api::{handle_db_error, handle_result, into_json_response},
  auth::{user::UserService, MyFirebaseUser},
  db::{self, engage::UpdateCouponSet},
  mezzofy,
};

pub async fn get(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage::get(&db, org_id, campaign_id, chain_id, &signer_address);

  handle_result(res.await)
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct ListParams {
  pub project_id: Option<Uuid>,
  pub campaign_id: Option<Uuid>,
  pub chain_id: Option<u64>,
  pub signer_address: Option<String>,
  pub user_id: Option<String>,
  pub created_after: Option<DateTime<Utc>>,
  pub created_before: Option<DateTime<Utc>>,
  pub order: Option<String>,
  pub offset: Option<u64>,
  pub limit: Option<u64>,
}

pub async fn list(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Query(p): Query<ListParams>,
  Path(org_id): Path<Uuid>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage::list(
    &db,
    db::engage::ListParams {
      org_id,
      project_id: p.project_id,
      campaign_id: p.campaign_id,
      chain_id: p.chain_id,
      signer_address: p.signer_address,
      user_id: p.user_id,
      created_after: p.created_after,
      created_before: p.created_before,
      order: p.order,
      offset: p.offset.unwrap_or_default(),
      limit: p.limit.unwrap_or(20u64),
    },
  );

  handle_result(res.await)
}

#[derive(Deserialize, Default)]
pub struct UpdateForm {
  pub accepted: db::engage::Accepted,
}

pub async fn approve(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id,  campaign_id, chain_id, signer_address)): Path<(
    Uuid,
    Uuid,
    i64,
    String,
  )>,
  Json(form): Json<UpdateForm>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage::approve(
    &db,
    org_id,
    campaign_id,
    chain_id,
    signer_address,
    form.accepted,
  );

  handle_result(res.await)
}

pub async fn delete(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage::delete(&db, org_id, campaign_id, chain_id, signer_address);

  handle_result(res.await)
}

pub async fn update_coupon(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
  Json(form): Json<UpdateCouponSet>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let res = db::engage::update_coupon(&db, org_id, campaign_id, chain_id, signer_address, form);

  handle_result(res.await)
}

#[skip_serializing_none]
#[derive(Serialize)]
pub struct EngageUpdated {
  coupon_issue_id: Option<String>,
  coupon_serial: Option<String>,
  coupon_url: Option<String>,
  updated_at: DateTime<Utc>,
}

pub async fn issue_coupon(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  State(mut mezzofy_client): State<mezzofy::Client>,
  State(mut user_service): State<UserService>,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
) -> Result<Response, Response> {
  if !user.can_edit(org_id) {
    return Err(StatusCode::FORBIDDEN.into_response());
  }

  let signer_address = &signer_address.to_lowercase();
  let en = db::engage::get(&db, org_id, campaign_id, chain_id, signer_address)
    .await
    .map_err(handle_db_error)?;
  if en.coupon_url.is_some() || en.coupon_serial.is_some() || en.coupon_issue_id.is_some() {
    return Err((StatusCode::CONFLICT, String::from("coupon already issued")).into_response());
  }

  match en.country_id {
    None => Err(failed_dep("Country not set")),
    Some(country_id) => match Country::from_id(country_id as u16) {
      None => Err(failed_dep("Country is set but not recognised")),
      Some(country) => {
        let u = user_service
          .lookup(&en.user_id)
          .await
          .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()).into_response())?;

        let customer = mezzofy::Customer {
          customer_id: &en.user_id,
          name: &u
            .displayName
            .unwrap_or(format!("{}/{}", chain_id, signer_address)),
          email: &u.email,
          // mobile_no: &phone_number,
          country_code: country.alpha2,
          join_date: Some(u.createdAt),
          first_name: None,
          last_name: None,
          address: None,
          dob: None,
          gender: None,
          reference_no: None,
        };

        let campaign = db::campaign::get(&db, org_id, campaign_id)
          .await
          .map_err(handle_db_error)?;
        match campaign.coupon_code {
          None => Err(failed_dep("Mezzofy coupon code missing")),
          Some(coupon_code) => {
            let mez_tx = mezzofy_client
              .issue_coupon(customer, &coupon_code)
              .await
              .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()).into_response())?;

            let up_res = db::engage::update(
              &db,
              org_id,
              campaign_id,
              chain_id,
              signer_address,
              db::engage::UpdateParam {
                coupon_issue_id: Some(&mez_tx.transaction_id),
                coupon_serial: None,
                coupon_url: None,
                country_id: None,
              },
            )
            .await
            .map_err(handle_db_error)?;

            Ok(into_json_response(&EngageUpdated {
              coupon_issue_id: Some(mez_tx.transaction_id),
              coupon_serial: None,
              coupon_url: None,
              updated_at: up_res.updated_at,
            }))
          }
        }
      }
    },
  }
}

fn failed_dep(msg: &str) -> Response {
  (StatusCode::FAILED_DEPENDENCY, String::from(msg)).into_response()
}

pub async fn commit_coupon(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  State(mut mezzofy_client): State<mezzofy::Client>,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
) -> Result<Response, Response> {
  if !user.can_edit(org_id) {
    return Err(StatusCode::FORBIDDEN.into_response());
  }

  let signer_address = &signer_address.to_lowercase();
  let en = db::engage::get(&db, org_id, campaign_id, chain_id, signer_address)
    .await
    .map_err(handle_db_error)?;
  if en.coupon_url.is_some() || en.coupon_serial.is_some() {
    return Err(
      (
        StatusCode::CONFLICT,
        String::from("coupon allready committed"),
      )
        .into_response(),
    );
  }
  match en.coupon_issue_id {
    Some(coupon_issue_id) => {
      let res = mezzofy_client
        .commit_coupon_issue(&coupon_issue_id)
        .await
        .map_err(|err| (StatusCode::BAD_GATEWAY, err.to_string()).into_response())?;

      let coupon = res.customer_coupons.into_iter().next().ok_or(
        (
          StatusCode::BAD_GATEWAY,
          String::from("Mezzofy customer coupons array is empty"),
        )
          .into_response(),
      )?;

      let up_res = db::engage::update(
        &db,
        org_id,
        campaign_id,
        chain_id,
        signer_address,
        db::engage::UpdateParam {
          coupon_issue_id: None,
          coupon_serial: Some(&coupon.customer_coupon.coupon_serial),
          coupon_url: None,
          country_id: None,
        },
      )
      .await
      .map_err(handle_db_error)?;

      Ok(into_json_response(&EngageUpdated {
        coupon_issue_id: None,
        coupon_serial: Some(coupon.customer_coupon.coupon_serial.to_owned()),
        coupon_url: None,
        updated_at: up_res.updated_at,
      }))
    }
    None => Err(
      (
        StatusCode::PRECONDITION_FAILED,
        String::from("coupon not yet issued"),
      )
        .into_response(),
    ),
  }
}

pub async fn delete_coupon_url(
  State(db): State<sqlx::PgPool>,
  user: MyFirebaseUser,
  Path((org_id, campaign_id, chain_id, signer_address)): Path<(Uuid, Uuid, i64, String)>,
) -> Response {
  if !user.can_edit(org_id) {
    return StatusCode::FORBIDDEN.into_response();
  }

  let signer_address = &signer_address.to_lowercase();
  match db::engage::delete_coupon_url(&db, org_id, campaign_id, chain_id, signer_address).await {
    Ok(res) => into_json_response(&EngageUpdated {
      coupon_issue_id: None,
      coupon_serial: None,
      coupon_url: None,
      updated_at: res.updated_at,
    }),
    Err(err) => handle_db_error(err),
  }
}
