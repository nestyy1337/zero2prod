use hyper::StatusCode;
use zero2prod::spawn_app;

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=lukar%20tim&email=lukar_tim%40gmail.com";
    let email = "lukar_tim@gmail.com";
    let _ = app.post_subscriptions(body.into()).await;

    let subscriber_data = sqlx::query!(r#"SELECT * FROM subscriptions WHERE email = $1"#, &email,)
        .fetch_one(&app.pool)
        .await
        .expect("subscriber should be set");

    let token = sqlx::query!(
        r#"SELECT subscription_tokens FROM subscriptions_tokens WHERE subscriber_id = $1"#,
        &subscriber_data.id
    )
    .fetch_one(&app.pool)
    .await
    .expect("token should be present")
    .subscription_tokens;

    assert_eq!(subscriber_data.status, "Pending");
    assert_eq!(subscriber_data.name, "lukar tim");
    assert_eq!(subscriber_data.email, "lukar_tim@gmail.com");

    app.get_confirm_subscription(&token).await;

    let subscriber_data = sqlx::query!(r#"SELECT * FROM subscriptions WHERE email = $1"#, &email,)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(subscriber_data.status, "confirmed");
    assert_eq!(subscriber_data.name, "lukar tim");
    assert_eq!(subscriber_data.email, "lukar_tim@gmail.com");
}

#[tokio::test]
async fn confirming_wrong_link_does_not_change_status() {
    let app = spawn_app().await;
    let body = "name=lukar%20tim&email=lukar_tim%40gmail.com";
    let email = "lukar_tim@gmail.com";
    let _ = app.post_subscriptions(body.into()).await;

    let subscriber_data = sqlx::query!(r#"SELECT * FROM subscriptions WHERE email = $1"#, &email,)
        .fetch_one(&app.pool)
        .await
        .expect("subscriber should be set");

    let token = sqlx::query!(
        r#"SELECT subscription_tokens FROM subscriptions_tokens WHERE subscriber_id = $1"#,
        &subscriber_data.id
    )
    .fetch_one(&app.pool)
    .await
    .expect("token should be present")
    .subscription_tokens;

    assert_eq!(subscriber_data.status, "Pending");
    assert_eq!(subscriber_data.name, "lukar tim");
    assert_eq!(subscriber_data.email, "lukar_tim@gmail.com");
    let reversed: String = token.chars().into_iter().rev().collect();
    app.get_confirm_subscription(&reversed).await;

    let subscriber_data = sqlx::query!(r#"SELECT * FROM subscriptions WHERE email = $1"#, &email,)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(subscriber_data.status, "Pending");
    assert_eq!(subscriber_data.name, "lukar tim");
    assert_eq!(subscriber_data.email, "lukar_tim@gmail.com");
}

#[tokio::test]
async fn bad_token_link_returns_500() {
    let app = spawn_app().await;
    let body = "name=lukar%20tim&email=lukar_tim%40gmail.com";
    let _ = app.post_subscriptions(body.into()).await;

    let token = "surely invalid token";

    let response = app.get_confirm_subscription(&token).await;

    assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
}
