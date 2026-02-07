-- Single-tenant Home Management System Schema (SQLite)

-- USERS (Family Members)
CREATE TABLE users (
    id BLOB PRIMARY KEY, -- Store UUID as BLOB
    username TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    birthday TEXT, -- SQLite uses TEXT for dates
    profile_picture_url TEXT,
    track_allowance INTEGER NOT NULL DEFAULT 0, -- SQLite uses 0/1 for boolean
    role TEXT NOT NULL DEFAULT 'member',  -- admin, member, child
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_users_username ON users(username);

-- ALLOWANCE LEDGER
CREATE TABLE allowance_ledger (
    seq INTEGER PRIMARY KEY AUTOINCREMENT, -- Use for deterministic ordering
    id BLOB NOT NULL UNIQUE,
    user_id BLOB NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    amount INTEGER NOT NULL, -- Store as cents
    balance INTEGER NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_allowance_ledger_user_id ON allowance_ledger(user_id);

-- CHORES
CREATE TABLE chores (
    id BLOB PRIMARY KEY,
    description TEXT NOT NULL,
    assigned_to BLOB REFERENCES users(id) ON DELETE SET NULL,
    reward INTEGER, -- Store as cents
    completed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_chores_assigned_to ON chores(assigned_to);

-- CALENDARS
CREATE TABLE calendars (
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT,
    google_id TEXT,
    color TEXT NOT NULL DEFAULT 'primary',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- DISPLAY TOKENS (for kiosk displays)
CREATE TABLE display_tokens (
    id BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_display_tokens_token ON display_tokens(token);

-- SETTINGS (key-value store)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- WEATHER CACHE
CREATE TABLE weather_cache (
    id INTEGER PRIMARY KEY CHECK (id = 1), -- Ensure only one row
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    data TEXT NOT NULL -- Store JSON as text
);

-- CALENDAR FEED CACHE (for iCal feeds)
CREATE TABLE calendar_feed_cache (
    calendar_id BLOB PRIMARY KEY REFERENCES calendars(id) ON DELETE CASCADE,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    ics_data TEXT NOT NULL
);

-- GOOGLE CALENDAR CACHE
CREATE TABLE google_calendar_cache (
    calendar_id BLOB PRIMARY KEY REFERENCES calendars(id) ON DELETE CASCADE,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    events TEXT NOT NULL -- Store JSON as text
);

-- TRiggers for updated_at
CREATE TRIGGER update_users_updated_at AFTER UPDATE ON users
BEGIN
    UPDATE users SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TRIGGER update_chores_updated_at AFTER UPDATE ON chores
BEGIN
    UPDATE chores SET updated_at = datetime('now') WHERE id = OLD.id;
END;

CREATE TRIGGER update_settings_updated_at AFTER UPDATE ON settings
BEGIN
    UPDATE settings SET updated_at = datetime('now') WHERE key = OLD.key;
END;