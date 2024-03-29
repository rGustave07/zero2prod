use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::{configuration, startup, telemetry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // <============= Logging initialization =============>
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // <============= DB / APP configuration =============>
    let config = configuration::get_configuration().expect("Failed to retrieve configuration");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    let address = format!("{}:{}", config.application.host, config.application.port);
    println!("{}", address.as_str());
    let listener = TcpListener::bind(address)?;

    // Application startup
    startup::run(listener, connection_pool)?.await
}
