use axum::{extract::State, response::Response};

use crate::{
  api::{handle_db_error, into_json_response},
  auth::MyFirebaseUser,
  db,
};

pub async fn get(
  user: MyFirebaseUser,
  State(db): State<sqlx::PgPool>,
) -> Result<Response, Response> {
  let xp = db::me::get_xp(&db, &user.sub)
    .await
    .map_err(handle_db_error)?;

  let club = db::me::get_club(&db, &user.sub)
    .await
    .map_err(handle_db_error)?;

  Ok(into_json_response(&db::me::Me { xp, club }))
}
