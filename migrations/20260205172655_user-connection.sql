CREATE TABLE user_connection(
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  from_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  to_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  is_accepted BOOLEAN NOT NULL DEFAULT FALSE,
  CONSTRAINT unique_from_to UNIQUE (from_id, to_id)
);

CREATE INDEX user_connection_from_id on user_connection (from_id);
CREATE INDEX user_connection_to_id on user_connection (to_id);
