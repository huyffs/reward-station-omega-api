pub mod firebase;
pub mod user;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
pub use firebase::ServiceAccount;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

pub type Orgs = HashMap<String, i64>;
pub type Wallets = HashMap<String, bool>;

pub const OWNER_PERMISSION: i64 = 0x1fffffffffffff;
pub const EDITOR_PERMISSION: i64 = 0x2;
// pub const VIEWER_PERMISSION: i64 = 0x1;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CustomClaims {
  #[serde(default)]
  pub admin: i64,
  #[serde(rename = "a", default)]
  pub orgs: Orgs,
  #[serde(rename = "w", default)]
  pub wallets: Wallets,
}

/// The Jwt claims decoded from the user token. Can also be viewed as the Firebase User
/// information.
#[derive(Deserialize, Clone)]
pub struct MyFirebaseUser {
  pub provider_id: Option<String>,
  pub name: Option<String>,
  pub picture: Option<String>,
  pub iss: String,
  pub aud: String,
  pub auth_time: u64,
  pub user_id: String,
  pub sub: String,
  pub iat: u64,
  pub exp: u64,
  pub email: Option<String>,
  pub email_verified: Option<bool>,
  #[serde(flatten)]
  pub claims: CustomClaims,
}

impl MyFirebaseUser {
  pub fn can_sudo(&self) -> bool {
    self.claims.admin > 0
  }

  pub fn can_edit(&self, org_id: Uuid) -> bool {
    let claim = self.claims.orgs.get(&org_id.to_string());
    matches!(claim, Some(p) if p.ge(&EDITOR_PERMISSION))
  }
/*   
  pub fn can_view(&self, org_id: Uuid) -> bool {
    let claim = self.claims.orgs.get(&org_id.to_string());
    matches!(claim, Some(p) if p.ge(&VIEWER_PERMISSION))
  }

  pub fn permission_level(&self, game_id: Uuid) -> i64 {
    match self.claims.orgs.get(&game_id.to_string()) {
      Some(p) => *p,
      None => 0,
    }
  }
 */
  pub fn has_wallet_claim(&self, chain_id: i64, signer_address: &str) -> bool {
    let id = format!("{}/{}", chain_id, signer_address);
    let claim = self.claims.wallets.get(&id);
    match claim {
      Some(c) => *c,
      _ => false,
    }
  }
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ProviderUserInfo {
  pub providerId: String,
  pub displayName: Option<String>,
  pub photoUrl: Option<String>,
  pub federatedId: Option<String>,
  pub email: Option<String>,
  pub rawId: String,
  pub screenName: Option<String>,
  pub phoneNumber: Option<String>,
}

#[serde_as]
#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct User {
  pub localId: String,
  pub email: String,
  pub displayName: Option<String>,
  pub language: Option<String>,
  pub photoUrl: Option<String>,
  pub timeZone: Option<String>,
  pub dateOfBirth: Option<String>,
  pub emailVerified: bool,
  pub passwordUpdatedAt: i64,
  pub providerUserInfo: Vec<ProviderUserInfo>,
  pub validSince: String,
  #[serde(default)]
  pub disabled: bool,
  #[serde(with = "serde_with::chrono_0_4::datetime_utc_ts_seconds_from_any")]
  pub lastLoginAt: DateTime<Utc>,
  #[serde(with = "serde_with::chrono_0_4::datetime_utc_ts_seconds_from_any")]
  pub createdAt: DateTime<Utc>,
  pub phoneNumber: Option<String>,
  #[serde_as(as = "serde_with::json::JsonString")]
  pub customAttributes: CustomClaims,
  #[serde(default)]
  pub emailLinkSignin: bool,
  pub initialEmail: Option<String>,
  pub lastRefreshAt: String,
}
