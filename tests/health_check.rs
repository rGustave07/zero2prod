use sqlx::{Connection, PgConnection, Row};
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let bound_address: String = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", bound_address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Could not retrieve config");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to database");

    let client = reqwest::Client::new();
    let body = "name=Ritz%20Gusty&email=ritzgusty%40gmail.com";
    let response = client
        .post(&format!("{}/subscribe", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ritzgusty@gmail.com");
    assert_eq!(saved.name, "Ritz Gusty");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Ritz%20Gusty", "missing the email"),
        ("email=ritzgusty%40gmail.com", "missing the name"),
        ("", "missing both email and name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 bad request when payload was {}.",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
