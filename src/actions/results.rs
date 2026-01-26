use std::fs;
use std::path::PathBuf;

use crate::model::PurgeData;

/// Filename for the most recent purge result
const LAST_RESULT_FILE: &str = "last_purge.json";

/// Resolve the cache path used by CLERGY
fn result_path() -> PathBuf {
    let mut dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    dir.push("clergy");

    // Best-effort directory creation
    let _ = fs::create_dir_all(&dir);

    dir.push(LAST_RESULT_FILE);
    dir
}

/// Persist the most recent successful purge
pub fn save_last(data: &PurgeData) -> Result<(), Box<dyn std::error::Error>> {
    let path = result_path();
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load the most recent purge result, if any
pub fn load_last() -> Option<PurgeData> {
    let path = result_path();
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}
