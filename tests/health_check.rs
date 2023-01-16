use actix_web::rt::spawn;
use sqlx::types::uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::collections::HashMap;
use std::net::TcpListener;
use uuid::Uuid;

use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::run;

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

#[actix_web::test]
async fn test_health_check() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let res = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to make request");
    assert!(res.status().is_success());
    assert_eq!(res.content_length(), Some(0));
}

#[actix_web::test]
async fn test_new_subscriber_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let mut data = HashMap::new();
    let test_email = "test@email.com";
    let test_name = "John Doe";
    data.insert("email", test_email);
    data.insert("name", test_name);

    let res = client
        .post(&format!("{}/subscriptions", &app.address))
        .form(&data)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(res.status().is_success());

    let new_sub = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(new_sub.name, test_name);
    assert_eq!(new_sub.email, test_email);
}

#[actix_web::test]
async fn test_new_subscriber_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=Ryan%20Papazoglou", "missing the email"),
        ("email=test%40email.com", "missing the name"),
        ("", "missing the email and the name"),
    ];

    for (invalid_body, error_message) in test_cases {
        let res = client
            .post(&format!("{}/subscriptions", &app.address))
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

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = get_configuration().expect("Failed to load configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let conn_pool = configure_database(&config.database).await;
    let server = run(listener, conn_pool.clone()).expect("Failed to start server.");
    let _ = spawn(server);

    TestApp {
        address,
        pool: conn_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    return connection_pool;
}
