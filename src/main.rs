use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{configuration, startup, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // <============= Logging initialization =============>
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);
    // <============= DB / APP configuration =============>
    let config = configuration::get_configuration().expect("Failed to retrieve configurations");
    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;
    // Application startup
    startup::run(listener, connection_pool)?.await
}
