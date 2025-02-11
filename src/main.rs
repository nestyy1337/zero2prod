use sqlx::PgPool;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::{get_subscriber, init_subscriber, run};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("TRACE", "zero2prod");
    init_subscriber(subscriber);

    let mut configuration = get_configuration().expect("Failed to read configuration.");

    let main_listener = tokio::net::TcpListener::bind(configuration.application.get_address())
        .await
        .expect("Failed to bind port 8000");

    configuration.database.database_name = "newsletter".to_string();
    let db = PgPool::connect_lazy_with(configuration.database.with_db());

    let _ = run(main_listener, db).await;
}
