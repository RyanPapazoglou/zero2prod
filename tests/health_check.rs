use actix_web::rt::spawn;
use sqlx::{Connection, PgConnection};
use std::collections::HashMap;
use std::net::TcpListener;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[actix_web::test]
async fn test_health_check() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let res = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to make request");
    assert!(res.status().is_success());
    assert_eq!(res.content_length(), Some(0));
}

#[actix_web::test]
async fn test_new_subscriber_200() {
    let address = spawn_app();
    let config = get_configuration().expect("Failed to load app config");
    let conn_string = config.database.connection_string();
    let mut conn = PgConnection::connect(&conn_string)
        .await
        .expect("Failed to connect to database");
    let client = reqwest::Client::new();
    let mut data = HashMap::new();
    let test_email = "test@email.com";
    let test_name = "John Doe";
    data.insert("email", test_email);
    data.insert("name", test_name);

    let res = client
        .post(&format!("{}/subscriptions", &address))
        .form(&data)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(res.status().is_success());

    let new_sub = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut conn)
        .await
        .expect("Failed to fetched saved subscription.");
    assert_eq!(new_sub.name, test_name);
    assert_eq!(new_sub.email, test_email);
}

#[actix_web::test]
async fn test_new_subscriber_400() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Ryan%20Papazoglou", "missing the email"),
        ("email=test%40email.com", "missing the name"),
        ("", "missing the email and the name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let res = client
            .post(&format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            res.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        )
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port.");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to start server.");
    let _ = spawn(server);
    format!("http://127.0.0.1:{}", port)
}
