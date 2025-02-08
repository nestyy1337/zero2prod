use lazy_static::lazy_static;
use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::{get_subscriber, init_subscriber, run};

lazy_static! {
    static ref SUBSCRIBER: () = {
        let default_filter = "info";
        let default_subscriber_name = "prod";

        let subscriber = get_subscriber(default_filter, default_subscriber_name);
        init_subscriber(subscriber);
    };
}

#[tokio::main]
async fn main() {
    lazy_static::initialize(&SUBSCRIBER);
    let main_listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Failed to bind port 8000");

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = "newsletter".to_string();
    let db = PgPool::connect_lazy(&configuration.database.connection_string())
        .expect("Failed to connect to main db");

    let _ = run(main_listener, db).await;
}
