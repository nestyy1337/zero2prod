use hyper::StatusCode;
use zero2prod::spawn_app;

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=luka%tim&email=luka_tim%40gmail.com";
    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn subscribe_return_500_unique_failed() {
    let test_app = spawn_app().await;

    let body = "name=luka%tim&email=luka_tim%40gmail.com";

    let _response = test_app.post_subscriptions(body.to_string()).await;
    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn subscribe_return_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_reason) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.to_string()).await;

        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_reason
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=luka%40gmail.com", "empty_name"),
        ("name=luka&email=", "empty email"),
        ("name=&email=", "both email and name empty"),
    ];

    for (body, error_reason) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not respond with 400 BAD_REQUEST but with: {} when the payload was: {}",
            response.status(),
            error_reason
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_name_is_long() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAa
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&email=luka%40gmail.com", "pretty fucking long name"),
    ];

    for (body, error_reason) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status(),
            "The API did not respond with 400 BAD_REQUEST but with: {} when the payload was: {}",
            response.status(),
            error_reason
        );
    }
}

async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _ = sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.pool)
        .await;
    let response = app.post_subscriptions(body.into()).await;
    assert_eq!(response.status().as_u16(), 500);
}
