use hyper::StatusCode;
use zero2prod::spawn_app;

#[tokio::test]
async fn health_check_test() {
    let app_address = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", &app_address))
        .send()
        .await
        .expect("Failed to execute get request");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.content_length(), Some(0));
}
