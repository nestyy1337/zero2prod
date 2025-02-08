use zero2prod::spawn_app;

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    let app_address = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=luka%tim&email=luka_tim%40gmail.com";
    let response = client
        .post(format!("{}/subscribe", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
}

#[tokio::test]
async fn subscribe_return_409_unique_failed() {
    let app_address = spawn_app().await;

    let client = reqwest::Client::new();
    let body = "name=luka%tim&email=luka_tim%40gmail.com";
    let _response = client
        .post(format!("{}/subscribe", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    let response = client
        .post(format!("{}/subscribe", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn subscribe_return_400_when_data_is_missing() {
    let app_address = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_reason) in test_cases {
        let response = client
            .post(format!("{}/subscribe", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_reason
        );
    }
}
