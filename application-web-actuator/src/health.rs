use application_web_macros::get;
use axum::response::IntoResponse;

#[get("/health/check")]
pub async fn health_check() -> impl IntoResponse {
    "Ok"
}
