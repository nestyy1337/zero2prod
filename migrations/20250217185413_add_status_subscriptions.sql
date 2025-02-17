-- Add migration script here
ALTER TABLE subscriptions ADD COLUMN status TEXT not null DEFAULT 'active';
