use axum::response::Response;

use serde::Serialize;

use crate::api::into_json_response;

#[derive(Serialize)]
pub struct Country<'a> {
  id: u16,
  name: &'a str,
  alpha2: &'a str,
}

pub async fn list() -> Response {
  let res = iso3166::countries::LIST
    .into_iter()
    .map(|c| Country {
      id: c.id,
      name: c.name,
      alpha2: c.alpha2,
    })
    .collect::<Vec<Country>>();

  into_json_response(&res)
}
