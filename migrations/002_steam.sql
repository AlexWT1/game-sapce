PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS store_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL UNIQUE,
    account_id TEXT NOT NULL,
    profile_url TEXT NOT NULL DEFAULT '',
    persona_name TEXT NOT NULL DEFAULT '',
    avatar_url TEXT NOT NULL DEFAULT '',
    api_key TEXT NOT NULL DEFAULT '',
    linked_at TEXT NOT NULL,
    last_synced_at TEXT
);

CREATE TABLE IF NOT EXISTS steam_games (
    game_id INTEGER NOT NULL PRIMARY KEY,
    appid INTEGER NOT NULL UNIQUE,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);
