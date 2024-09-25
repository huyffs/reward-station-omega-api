use axum::{
  async_trait,
  extract::{FromRef, FromRequestParts, State},
  http::{request::Parts, StatusCode},
  response::{IntoResponse, Response},
  routing::{delete, get, patch, post},
  Router,
};
use axum_extra::{
  headers::{authorization::Bearer, Authorization},
  TypedHeader,
};
use firebase_auth::FirebaseAuth;
use serde::Serialize;

use crate::{
  auth::{user::UserService, MyFirebaseUser},
  db::{self},
  mezzofy, subscan,
};

use self::rs::engage::EngageEventStream;

pub mod cm;
pub mod rs;
pub mod su;

#[derive(Clone)]
pub struct AppState {
  pub pool: sqlx::PgPool,
  pub firebase_auth: FirebaseAuth,
  pub claims_service: UserService,
  pub engage_event_stream: EngageEventStream,
  pub mezzofy_client: mezzofy::Client,
  pub subscan_client: subscan::Client,
}

impl FromRef<AppState> for sqlx::PgPool {
  fn from_ref(state: &AppState) -> Self {
    state.pool.clone()
  }
}

pub struct Server {
  pub router: Router,
}

impl Server {
  pub fn new(
    pool: sqlx::PgPool,
    firebase_auth: FirebaseAuth,
    claims_service: UserService,
    engage_event_stream: EngageEventStream,
    mezzofy_client: mezzofy::Client,
    subscan_client: subscan::Client,
  ) -> Self {
    let app_state = AppState {
      pool,
      firebase_auth,
      claims_service,
      engage_event_stream,
      mezzofy_client,
      subscan_client,
    };

    let router = axum::Router::new()
      .route("/", get(home))
      .route("/health", get(health))
      // .route("/su/init", post(su::auth::init))
      .route("/su/grant", post(su::auth::grant))
      .route(
        "/su/project-reward/:org_id/:project_id/:reward_id",
        patch(su::project_reward::update),
      )
      .route(
        "/su/campaign-reward/:org_id/:campaign_id/:reward_id",
        patch(su::campaign_reward::update),
      )
      .route("/su/coupon", get(su::coupon::list).post(su::coupon::create))
      .route(
        "/su/coupon/:reward_id/:number",
        get(su::coupon::get)
          .patch(su::coupon::update)
          .put(su::coupon::replace)
          .delete(su::coupon::delete),
      )
      .route("/cm/grant/:org_id", get(cm::auth::grant))
      .route("/cm/org", get(cm::org::list).post(cm::org::create))
      .route(
        "/cm/org/:org_id",
        get(cm::org::get)
          .patch(cm::org::update)
          .put(cm::org::replace)
          .delete(cm::org::delete),
      )
      .route(
        "/cm/project/:org_id",
        get(cm::project::list).post(cm::project::create),
      )
      .route(
        "/cm/project/:org_id/:project_id",
        get(cm::project::get)
          .patch(cm::project::update)
          .put(cm::project::replace)
          .delete(cm::project::delete),
      )
      .route("/cm/reward", get(cm::reward::list).post(cm::reward::create))
      .route(
        "/cm/reward/:reward_id",
        get(cm::reward::get)
          .patch(cm::reward::update)
          .put(cm::reward::replace)
          .delete(cm::reward::delete),
      )
      .route(
        "/cm/project-reward/:org_id/:project_id",
        get(cm::project_reward::list).post(cm::project_reward::create),
      )
      .route(
        "/cm/project-reward/:org_id/:project_id/:reward_id",
        get(cm::project_reward::get)
          .patch(cm::project_reward::update)
          .delete(cm::project_reward::unlink),
      )
      .route(
        "/cm/campaign/:org_id",
        get(cm::campaign::list).post(cm::campaign::create),
      )
      .route(
        "/cm/campaign/:org_id/:campaign_id",
        get(cm::campaign::get)
          .patch(cm::campaign::update)
          .put(cm::campaign::replace)
          .delete(cm::campaign::delete),
      )
      .route(
        "/cm/campaign-reward/:org_id/:campaign_id",
        get(cm::campaign_reward::list).post(cm::campaign_reward::create),
      )
      .route(
        "/cm/campaign-reward/:org_id/:campaign_id/:reward_id",
        get(cm::campaign_reward::get)
          .patch(cm::campaign_reward::update)
          .delete(cm::campaign_reward::unlink),
      )
      .route("/cm/engage/:org_id", get(cm::engage::list))
      .route(
        "/cm/engage/:org_id/:campaign_id/:chain_id/:signer_address",
        get(cm::engage::get)
          .patch(cm::engage::approve)
          .delete(cm::engage::delete),
      )
      .route(
        "/cm/engage/:org_id/:campaign_id/:chain_id/:signer_address/coupon",
        patch(cm::engage::update_coupon),
      )
      .route(
        "/cm/engage/:org_id/:campaign_id/:chain_id/:signer_address/coupon_url",
        delete(cm::engage::delete_coupon_url),
      )
      .route(
        "/cm/engage/:org_id/:campaign_id/:chain_id/:signer_address/issue",
        post(cm::engage::issue_coupon),
      )
      .route(
        "/cm/engage/:org_id/:campaign_id/:chain_id/:signer_address/commit",
        post(cm::engage::commit_coupon),
      )
      .route("/countries", get(rs::country::list))
      .route("/auth/fix", get(rs::auth::fix_claims))
      .route("/auth/link", post(rs::auth::link))
      .route("/projects", get(rs::project::list))
      .route("/projects/:project_id", get(rs::project::get))
      .route("/clubs", get(rs::club::list_my_clubs).post(rs::club::join))
      .route("/clubs/:project_id", get(rs::voucher::get_project_point))
      .route("/engage", get(rs::campaign::list))
      .route("/engage/:campaign_id", get(rs::campaign::get))
      .route("/engage/:campaign_id/org_id", get(rs::campaign::get_org_id))
      .route(
        "/tasks/:chain_id/:signer_address",
        get(rs::engage::get_tasks),
      )
      .route(
        "/tasks/:chain_id/:signer_address/:campaign_id",
        get(rs::engage::get)
          .post(rs::engage::create)
          .patch(rs::engage::update)
          .put(rs::engage::submit_proof),
      )
      .route("/coupons", get(rs::coupon::list))
      .route("/coupons/:reward_id/:number", get(rs::coupon::get))
      // .route(
      //   "/coupons/:chain_id/:signer_address/:campaign_id",
      //   get(rs::engage::retrieve_coupon),
      // )
      .route("/events", get(rs::engage::list_events))
      .route("/events/stream", get(rs::engage::events))
      .route("/nft/:chain_id/:signer_address", get(rs::nft::list))
      .route("/rewards", get(rs::reward::list))
      .route("/rewards/:reward_id", get(rs::reward::get))
      .route(
        "/project-rewards/:project_id",
        get(rs::project_reward::list),
      )
      .route(
        "/project-rewards/:project_id/:reward_id",
        get(rs::project_reward::get).post(rs::reward::mint_from_project),
      )
      .route(
        "/campaign-participations/:campaign_id",
        get(rs::campaign_participation::list),
      )
      .route(
        "/campaign-rewards/:campaign_id",
        get(rs::campaign_reward::list),
      )
      .route(
        "/campaign-rewards/:campaign_id/:reward_id",
        get(rs::campaign_reward::get).post(rs::reward::mint_from_campaign),
      )
      .route("/vouchers", get(rs::voucher::list))
      .route(
        "/vouchers/:campaign_id/:chain_id/:signer_address/:task_id",
        get(rs::voucher::get),
      )
      .route("/me", get(rs::me::get))
      .with_state(app_state);

    Self { router }
  }
}

// home
async fn home() -> &'static str {
  "Hello, World!"
}

// check health
async fn health(State(db): State<sqlx::PgPool>) -> (StatusCode, &'static str) {
  match db::health(&db).await {
    Ok(()) => (StatusCode::OK, "👍 Healthy!"),
    _ => (StatusCode::INTERNAL_SERVER_ERROR, "😭 Degraded!"),
  }
}

pub fn handle_db_error(err: db::Error) -> Response {
  match err {
    db::Error::EmptyUpdateSet | db::Error::InvalidOrder => {
      (StatusCode::BAD_REQUEST, err.to_string()).into_response()
    }
    db::Error::NotFound => StatusCode::NOT_FOUND.into_response(),
    _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
  }
}

pub fn handle_result<T: Serialize>(res: Result<T, db::Error>) -> Response {
  match res {
    Ok(data) => into_json_response(&data),
    Err(err) => handle_db_error(err),
  }
}

pub fn into_json_response<T: Serialize>(data: &T) -> Response {
  serde_json::to_string(data).unwrap().into_response()
}

#[async_trait]
impl<S> FromRequestParts<S> for MyFirebaseUser
where
  S: Send + Sync,
  AppState: FromRef<S>,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) =
      TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
        .await
        .map_err(http_error_handler(StatusCode::BAD_REQUEST))?;

    AppState::from_ref(state)
      .firebase_auth
      .verify(bearer.token())
      .map_err(|err| (StatusCode::UNAUTHORIZED, err.to_string()))
  }
}

#[derive(Clone)]
pub struct MaybeUser(pub Option<MyFirebaseUser>);

#[async_trait]
impl<S> FromRequestParts<S> for MaybeUser
where
  S: Send + Sync,
  AppState: FromRef<S>,
{
  type Rejection = (StatusCode, String);

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let TypedHeader(Authorization(bearer)) =
      match TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await {
        Ok(b) => b,
        Err(_) => return Ok(Self(None)),
      };

    match AppState::from_ref(state)
      .firebase_auth
      .verify(bearer.token())
    {
      Ok(user) => Ok(Self(Some(user))),
      Err(err) => Err((StatusCode::UNAUTHORIZED, err.to_string())),
    }
  }
}

fn http_error_handler<E>(status: StatusCode) -> impl Fn(E) -> (StatusCode, String)
where
  E: std::error::Error,
{
  move |err: E| -> (StatusCode, String) { (status, err.to_string()) }
}

impl FromRef<AppState> for FirebaseAuth {
  fn from_ref(state: &AppState) -> Self {
    state.firebase_auth.clone()
  }
}

impl FromRef<AppState> for mezzofy::Client {
  fn from_ref(state: &AppState) -> Self {
    state.mezzofy_client.clone()
  }
}

impl FromRef<AppState> for subscan::Client {
  fn from_ref(state: &AppState) -> Self {
    state.subscan_client.clone()
  }
}

impl FromRef<AppState> for EngageEventStream {
  fn from_ref(state: &AppState) -> Self {
    state.engage_event_stream.clone()
  }
}

pub struct UnauthorizedResponse {
  msg: String,
}

impl IntoResponse for UnauthorizedResponse {
  fn into_response(self) -> Response {
    (StatusCode::UNAUTHORIZED, self.msg).into_response()
  }
}

impl FromRef<AppState> for UserService {
  fn from_ref(state: &AppState) -> Self {
    state.claims_service.clone()
  }
}
