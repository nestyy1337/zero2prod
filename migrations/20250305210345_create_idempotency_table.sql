-- Add migration script here
CREATE TYPE EmailStatus AS ENUM ('Pending', 'Sent', 'Failed');

CREATE TABLE idempotency (
  user_id uuid NOT NULL REFERENCES subscriptions(id),
  idempotency_key TEXT NOT NULL,
  status EmailStatus NOT NULL,
  attempts INT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (user_id, idempotency_key)
);
