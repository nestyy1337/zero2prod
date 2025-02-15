use axum::response::IntoResponse;

pub async fn health_check() -> impl IntoResponse {
    tracing::info!("Successful healthcheck");
    String::from("App is healthy!")
}
