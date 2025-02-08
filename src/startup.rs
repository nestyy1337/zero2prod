use crate::{
    configuration::get_configuration,
    routes::{health_check::health_check, subscriptions::subscribe},
};
use axum::{
    Router,
    body::Body,
    extract::Request,
    routing::{get, post},
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};
use uuid::Uuid;

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<String, std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    tracing::info!(
        "Connected to database with string: {}",
        &configuration.database.connection_string()
    );

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(pool)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = Uuid::new_v4();
                    tracing::span!(
                        Level::INFO,
                        "request: ",
                        method = tracing::field::display(request.method()),
                        uri = tracing::field::display(request.uri()),
                        version = tracing::field::debug(request.version()),
                        request_id = tracing::field::display(request_id)
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    tracing::info!("Successfully created router");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    tracing::info!("About to start listening on: {}", &address);

    axum::serve(listener, app).await?;

    Ok(address)
}

pub fn get_subscriber(name: &str, filter: &str) -> impl Subscriber + Send + Sync {
    let formatting = BunyanFormattingLayer::new(name.into(), std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    let subsciber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting);
    subsciber
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set global subscriber");
}
