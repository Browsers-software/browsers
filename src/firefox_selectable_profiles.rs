use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use tracing::info;

// Returns (path, name) of each profile in a Profile Groups/<StoreID>.sqlite.
// Path is relative to the Firefox root dir (or absolute).
pub fn read_selectable_profiles(db_path: &Path) -> Vec<(String, String)> {
    if !db_path.exists() {
        info!("Selectable profiles db '{}' does not exist", db_path.display());
        return Vec::new();
    }
    let result = (|| -> rusqlite::Result<Vec<(String, String)>> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let mut stmt = conn.prepare("SELECT path, name FROM Profiles ORDER BY id")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect()
    })();
    match result {
        Ok(profiles) => profiles,
        Err(e) => {
            info!(
                "Could not read selectable profiles from '{}': {}",
                db_path.display(),
                e
            );
            Vec::new()
        }
    }
}
