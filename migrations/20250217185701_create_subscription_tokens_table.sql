-- Add migration script here
CREATE TABLE subscriptions_tokens(
    subscription_tokens TEXT not null,
    subscriber_id uuid NOT NULL
        REFERENCES subscriptions (id),
    PRIMARY KEY (subscription_tokens)
);

