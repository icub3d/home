-- API Keys
CREATE TABLE api_keys (
    id BLOB PRIMARY KEY, -- UUID
    key_hash TEXT NOT NULL UNIQUE, -- Hashed key for lookup
    name TEXT NOT NULL, -- Human readable name
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_used_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
