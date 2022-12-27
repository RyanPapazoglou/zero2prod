use actix_web::rt::spawn;
use std::net::TcpListener;

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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port.");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::run(listener).expect("Failed to start server.");
    let _ = spawn(server);
    format!("http://127.0.0.1:{}", port)
}
