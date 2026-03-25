mod models;
mod schemas;
mod handlers;


use actix_web::{web};
use utoipa::{OpenApi, openapi};

pub const SUB_ROUTE: &str = "/accounts";

pub fn get_router() -> actix_web::Scope {
    web::scope(SUB_ROUTE)
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
    path =  format!("/profile"),
    security(("bearer_auth" = []))  // 👈 applied per endpoint
)]
#[actix_web::get("/profile")]
async fn profile() -> String {
    "This is the profile page".to_string()
}
