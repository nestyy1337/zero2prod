use crate::{
    configuration::get_configuration,
    email_client::EmailClient,
    routes::{confirm::confirm_subscriber, health_check::health_check, subscriptions::subscribe},
};
use aws_config::BehaviorVersion;
use aws_sdk_ses::Client;
use axum::{
    body::Body,
    extract::Request,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{subscriber::set_global_default, Level, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub email_client: EmailClient,
}
impl AppState {
    pub fn new(pool: PgPool, client: EmailClient) -> Self {
        Self {
            pool,
            email_client: client,
        }
    }
}

pub async fn run(listener: TcpListener, pool: PgPool) -> Result<String, std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    tracing::info!(
        "Connected to database with string: {:?}",
        configuration.database.with_db()
    );

    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let client = EmailClient::new(Client::new(&config));

    let app_state = AppState::new(pool, client);

    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .route("/subscribe/confirm", get(confirm_subscriber))
        .with_state(app_state)
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

    let address = configuration.application.get_address();
    tracing::info!("About to start listening on: {}", &address);

    axum::serve(listener, app).await?;

    Ok(address)
}

pub fn get_subscriber(name: &str, filter: &str) -> impl Subscriber + Send + Sync {
    let formatting = BunyanFormattingLayer::new(name.into(), std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into());
    println!("{:?}", env_filter);

    let subsciber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting);
    subsciber
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set global subscriber");
}
