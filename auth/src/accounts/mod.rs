mod models;
mod schemas;


use actix_web::{web};
use utoipa::{OpenApi, openapi};

const sub_route: &str = "/accounts";

pub fn get_router() -> actix_web::Scope {
    web::scope(sub_route)
    .service(profile)
}

#[derive(OpenApi)]
#[openapi(
    paths(profile),
    components(
        schemas()

    )
)]
pub struct AccountsApiDoc;


#[utoipa::path(
    get,
    path =  format!("{}/profile", sub_route),
    security(("bearer_auth" = []))  // 👈 applied per endpoint
)]
#[actix_web::get("/profile")]
async fn profile() -> String {
    "This is the profile page".to_string()
}