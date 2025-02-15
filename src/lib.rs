use configuration::{configure_database, get_configuration};
use lazy_static::lazy_static;
use sqlx::PgPool;
use tokio::net::TcpListener;

use startup::{get_subscriber, init_subscriber, run};
use uuid::Uuid;

pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod startup;

lazy_static! {
    static ref SUBSCRIBER: () = {
        let default_filter = "TRACE";
        let default_subscriber_name = "test";

        let subscriber = get_subscriber(default_filter, default_subscriber_name);
        init_subscriber(subscriber);
    };
}

pub async fn spawn_app() -> String {
    lazy_static::initialize(&SUBSCRIBER);
    let test_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind the listener");

    let mut configuration = get_configuration().expect("Failed to get configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    configure_database(&configuration.database).await;
    let db = PgPool::connect_lazy_with(configuration.database.with_db());

    let port = test_listener.local_addr().unwrap().port();
    let server = run(test_listener, db);
    tokio::spawn(server);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    tracing::info!(
        "test addr: {}",
        format!("http://127.0.0.1:{}", port.to_string()),
    );
    format!("http://127.0.0.1:{}", port.to_string())
}
