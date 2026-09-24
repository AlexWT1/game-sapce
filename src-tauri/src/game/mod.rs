pub mod achievements;
pub mod launcher;
pub mod seed;
pub mod theme;

/// Formats seconds as `H:MM:SS` (tray / compact display).
pub fn format_hms(total: i64) -> String {
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
