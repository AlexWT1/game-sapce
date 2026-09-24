PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    executable_path TEXT,
    install_path TEXT,
    cover_path TEXT,
    background_path TEXT,
    icon_path TEXT,
    developer TEXT NOT NULL DEFAULT '',
    publisher TEXT NOT NULL DEFAULT '',
    genre TEXT NOT NULL DEFAULT '',
    release_year INTEGER,
    platform TEXT NOT NULL DEFAULT 'PC',
    source TEXT NOT NULL DEFAULT 'Manual',
    controller_support TEXT NOT NULL DEFAULT 'unknown',
    is_installed INTEGER NOT NULL DEFAULT 0,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    is_hidden INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_played_at TEXT,
    total_playtime_seconds INTEGER NOT NULL DEFAULT 0,
    launch_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_games_title ON games(title);
CREATE INDEX IF NOT EXISTS idx_games_last_played ON games(last_played_at);

CREATE TABLE IF NOT EXISTS game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    pid INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_game ON game_sessions(game_id);
CREATE INDEX IF NOT EXISTS idx_sessions_started ON game_sessions(started_at);

CREATE TABLE IF NOT EXISTS achievements (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon_path TEXT,
    rarity TEXT NOT NULL DEFAULT 'Common',
    unlocked INTEGER NOT NULL DEFAULT 0,
    unlocked_at TEXT,
    progress_current INTEGER NOT NULL DEFAULT 0,
    progress_target INTEGER NOT NULL DEFAULT 1,
    scope TEXT NOT NULL DEFAULT 'game',
    key TEXT,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_achievements_key ON achievements(key) WHERE key IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_achievements_game ON achievements(game_id);

CREATE TABLE IF NOT EXISTS game_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_game_tags_unique ON game_tags(game_id, tag);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS game_themes (
    game_id INTEGER PRIMARY KEY,
    primary_color TEXT NOT NULL DEFAULT '#7c5cff',
    secondary_color TEXT NOT NULL DEFAULT '#2dd4bf',
    accent_color TEXT NOT NULL DEFAULT '#a78bfa',
    background_color TEXT NOT NULL DEFAULT '#0a0c1c',
    glow_color TEXT NOT NULL DEFAULT '#7c5cff',
    font_family TEXT NOT NULL DEFAULT 'Inter',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);
