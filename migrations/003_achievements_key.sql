-- A game-scoped achievement key (e.g. first_launch_game) must be unique per game,
-- not globally: otherwise only the very first created game can receive it and every
-- later insert (manual create or Steam sync) fails with a UNIQUE constraint error.
DROP INDEX IF EXISTS idx_achievements_key;

CREATE UNIQUE INDEX IF NOT EXISTS idx_achievements_key
ON achievements(COALESCE(game_id, 0), key) WHERE key IS NOT NULL;
