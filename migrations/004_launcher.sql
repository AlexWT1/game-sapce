-- Games that must be started through their own launcher.
-- `use_launcher` switches the launch flow to `launcher_path`;
-- `is_installed` for such games is verified against the launcher executable.
ALTER TABLE games ADD COLUMN use_launcher INTEGER NOT NULL DEFAULT 0;
ALTER TABLE games ADD COLUMN launcher_path TEXT;
