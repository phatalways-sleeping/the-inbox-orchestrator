-- Add migration script here
ALTER TABLE issue_delivery_table
ADD COLUMN n_retries SMALLINT NOT NULL DEFAULT 0;

ALTER TABLE issue_delivery_table
ADD COLUMN execute_after timestamptz NOT NULL DEFAULT now();