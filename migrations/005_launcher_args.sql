-- Command-line arguments passed to the game's launcher on start (optional).
ALTER TABLE games ADD COLUMN launcher_args TEXT NOT NULL DEFAULT '';