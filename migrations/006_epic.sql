PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS epic_games (
    game_id INTEGER NOT NULL PRIMARY KEY,
    app_name TEXT NOT NULL UNIQUE,
    catalog_item_id TEXT NOT NULL DEFAULT '',
    namespace TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);
