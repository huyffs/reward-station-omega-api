use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Utc};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};

use crate::rec_http;

#[derive(Debug, Serialize, Clone)]
struct Coupon<'a> {
  coupon_code: &'a str,
  coupon_count: &'a str,
}

#[derive(Debug, Serialize, Clone)]
pub struct Customer<'a> {
  pub customer_id: &'a str,
  pub name: &'a str,
  pub email: &'a str,
  // pub mobile_no: &'a str,
  pub country_code: &'a str,
  pub join_date: Option<DateTime<Utc>>,
  pub first_name: Option<&'a str>,
  pub last_name: Option<&'a str>,
  pub address: Option<&'a str>,
  pub dob: Option<&'a str>,
  pub gender: Option<&'a str>,
  pub reference_no: Option<&'a str>,
}

#[derive(Debug, Serialize, Clone)]
struct CouponIssue<'a> {
  customer: Customer<'a>,
  outlet_id: &'a str,
  delivery_method: &'a str,
  coupon_delivery_type: &'a str,
  // purchase_receipt: &'a str,
  // receipt_template_id: &'a str,
  // template_id: &'a str,
  payment_name: &'a str,
  transaction_amount: i64,
  lang: &'a str,
  coupons: Vec<Coupon<'a>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponIssueResult {
  pub transaction_id: String,
  pub updated_on: DateTime<Utc>,
  pub po_status: String,
  pub po_date: DateTime<Utc>,
  pub pay_response: String,
  pub merchant_id: String,
  pub pay_receipt: Option<String>,
  pub coupon_via: String,
  pub po_total: i64,
  pub hash_code: String,
  pub created_on: DateTime<Utc>,
  pub reward_point: Option<String>,
  pub payment_name: Option<String>,
  pub customer_id: String,
  pub po_no: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CouponIssueCommit<'a> {
  pub delivery_method: &'a str,
  pub lang: &'a str,
  pub coupon_delivery_type: &'a str,
  pub payment_name: Option<&'a str>,
  pub pay_receipt: Option<&'a str>,
  pub purchase_receipt: &'a str,
  pub receipt_template_id: &'a str,
  pub template_id: &'a str,
  pub transaction_gateway: &'a str,
  pub transaction_amount: &'a str,
  pub transaction_note: &'a str,
  pub reference_no: Option<&'a str>,
  pub issue_reference: Option<&'a str>,
  pub transaction_ref_no: Option<&'a str>,
  pub processed_by: Option<&'a str>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponIssueCommitResult {
  // pub transaction_id: &'a str,
  // pub merchant_id: &'a str,
  // pub customer_id: &'a str,
  // pub coupon_via: &'a str,
  // pub payment_name: Option<&'a str>,
  // pub pay_receipt: Option<&'a str>,
  // pub pay_response: &'a str,
  // pub po_no: &'a str,
  // pub po_date: &'a str,
  // pub po_total: i64,
  // pub po_status: &'a str,
  // pub reward_point: Option<i64>,
  // pub hash_code: &'a str,
  // pub created_on: DateTime<Utc>,
  // pub updated_on: DateTime<Utc>,
  pub customer_coupons: Vec<CustomerCoupons>,
}
/*
#[derive(Debug, Serialize, Clone)]
struct TxRedeem<'a> {
  outlet_id: &'a str,
  lang: &'a str,
  transaction_ref_no: Option<&'a str>,
  transaction_note: Option<&'a str>,
}

#[derive(Debug, Serialize, Clone)]
struct TxSerial<'a> {
  serial: &'a str,
  redeem_value: f64,
}

#[derive(Debug, Serialize, Clone)]
struct CouponRedeem<'a> {
  txn_redeem: TxRedeem<'a>,
  txn_serials: TxSerial<'a>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponRedeemResult {
  pub transaction_id: &'a str,
  pub outlet_id: &'a str,
  pub merchant_id: &'a str,
  pub user_type: &'a str,
  pub transaction_type: &'a str,
  pub transaction_status: &'a str,
  pub lang: &'a str,
  pub updated_on: NaiveDateTime,
  pub created_on: NaiveDateTime,
}

#[derive(Debug, Serialize, Clone)]
pub struct CouponRedeemCommit<'a> {
  transaction_by: Option<&'a str>,
  transaction_note: Option<&'a str>,
  transaction_ref_no: Option<&'a str>,
  reference_image: Option<&'a str>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponRedeemCommitResponse<'a> {
  pub outlet_id: &'a str,
  pub operator_id: &'a str,
  pub merchant_id: &'a str,
  pub customer_id: &'a str,
  pub order_date: &'a str,
  pub order_no: &'a str,
  pub order_status: &'a str,
  pub order_type: &'a str,
  pub pos_redemption_flag: &'a str,
  pub pos_redemption_qr_code: Option<&'a str>,
  pub pos_redemption_bar_code: Option<&'a str>,
  pub pos_redemption_code: Option<&'a str>,
  pub guestckId: &'a str,
  pub branchCode: &'a str,
  pub lang: &'a str,
  pub hash_code: i64,
  pub updated_on: NaiveDateTime,
  pub created_on: NaiveDateTime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponRedeemCommitResult<'a> {
  pub txn_serials: Vec<&'a str>,
  pub redeem_response: CouponRedeemCommitResponse<'a>,
}
 */

#[derive(Debug, Deserialize, Clone)]
pub struct CustomerCoupons {
  pub customer_coupon: CustomerCoupon,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CustomerCoupon {
  // pub coupon_no: String,
  pub coupon_serial: String,
  // pub coupon_code: String,
  // pub coupon_name: String,
  // pub product_desc: String,
  // pub orginal_price: Option<f64>,
  // pub selling_price: Option<f64>,
  // pub purchase_date: DateTime<Utc>,
  // pub start_date: DateTime<Utc>,
  // pub end_date: DateTime<Utc>,
  // pub coupon_status: String,
  // pub updated_on: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Serial {
  pub single_serial_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponSerial {
  pub serial: Serial,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CouponSerialResult {
  pub couponserial: CouponSerial,
}

#[derive(Debug, Deserialize, Clone)]
struct IdToken {
  access_token: String,
  access_token_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Client {
  cred: String,
  auth_header: String,
  id_token_expiry: chrono::DateTime<Utc>,
  http_client: rec_http::Client,
}

impl Client {
  pub fn new(api_key: &str, secret: &str, http_client: rec_http::Client) -> Self {
    Self {
      cred: format!(
        "Basic {}",
        general_purpose::STANDARD.encode(format!("{}:{}", api_key, secret))
      ),
      auth_header: String::from(""),
      id_token_expiry: chrono::offset::Utc::now(),
      http_client,
    }
  }

  async fn fetch_id_token(&self) -> Result<IdToken> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, self.cred.parse().unwrap());
    headers.insert(
      CONTENT_TYPE,
      "application/x-www-form-urlencoded".parse().unwrap(),
    );
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    let text = self
      .http_client
      .post_form(
        "https://auth.mzapiea.mezzofy.com/v2/token",
        headers,
        &HashMap::from([("grant_type", "client_credentials")]),
      )
      .await?;

    match text {
      Some(s) => serde_json::from_str(&s).map_err(|err| anyhow!(err)),
      None => bail!("Empty response body"),
    }
  }

  async fn get_auth_header(&mut self) -> Result<()> {
    let now = chrono::offset::Utc::now();
    if self.id_token_expiry < now || self.auth_header.is_empty() {
      let id_token = self.fetch_id_token().await?;
      self.auth_header = format!("Bearer {}", &id_token.access_token);
      self.id_token_expiry = id_token.access_token_expires_at;
    }
    Ok(())
  }

  pub async fn issue_coupon<'a>(
    &mut self,
    customer: Customer<'a>,
    coupon_code: &'a str,
  ) -> Result<CouponIssueResult> {
    self.get_auth_header().await?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, self.auth_header.parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let text = self
      .http_client
      .post_json(
        "https://transaction.mzapi.mezzofy.com/v2/issue",
        headers,
        &CouponIssue {
          customer,
          delivery_method: "I",
          lang: "en",
          outlet_id: "8WCLI",
          payment_name: "Braintree",
          transaction_amount: 1,
          coupon_delivery_type: "I",
          coupons: vec![Coupon {
            coupon_code,
            coupon_count: "1",
          }],
        },
      )
      .await?;

    match text {
      Some(s) => serde_json::from_str(&s).map_err(|err| anyhow!(err)),
      None => bail!("Empty response body"),
    }
  }

  pub async fn commit_coupon_issue<'a>(
    &mut self,
    transaction_id: &'a str,
  ) -> Result<CouponIssueCommitResult> {
    self.get_auth_header().await?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, self.auth_header.parse().unwrap());
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    let text = self
      .http_client
      .post_json(
        &format!(
          "https://transaction.mzapi.mezzofy.com/v2/issue/{}/commit",
          transaction_id
        ),
        headers,
        &CouponIssueCommit {
          delivery_method: "I",
          lang: "en",
          purchase_receipt: "N",
          template_id: "MEZZOFY_MRMDR",
          receipt_template_id: "Mezzofy",
          transaction_gateway: "stripe",
          transaction_note: "No note",
          issue_reference: None,
          pay_receipt: None,
          processed_by: None,
          reference_no: None,
          transaction_ref_no: None,
          payment_name: None,
          transaction_amount: "1",
          coupon_delivery_type: "I",
        },
      )
      .await?;

    match text {
      Some(s) => serde_json::from_str(&s).map_err(|err| anyhow!(err)),
      None => bail!("Empty response body"),
    }
  }

  pub async fn get_coupon<'a>(&mut self, serial: &'a str) -> Result<CouponSerialResult> {
    self.get_auth_header().await?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(AUTHORIZATION, self.auth_header.parse().unwrap());

    let text = self
      .http_client
      .get(
        &format!("https://serial.mzapiea.mezzofy.com/v2/{}", serial),
        headers,
      )
      .await?;

    match text {
      Some(s) => serde_json::from_str(&s).map_err(|err| anyhow!(err)),
      None => bail!("Empty response body"),
    }
  }
}
