use config::{Config, File, FileFormat};
use serde::Deserialize;
use sqlx::Executor;
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool =
        PgPool::connect_lazy(&config.connection_string()).expect("Failed to get PgPool");
    sqlx::migrate!()
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let config = Config::builder()
        .add_source(File::new("./config/configuration.yaml", FileFormat::Yaml))
        .build()?;
    config.try_deserialize()
}
