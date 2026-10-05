use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

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

/// Persist the most recent run, including partial failures, without torn writes.
pub fn save_last(data: &PurgeData) -> Result<(), Box<dyn std::error::Error>> {
    let path = result_path();
    let json = serde_json::to_string_pretty(data)?;
    let mut file = NamedTempFile::new_in(path.parent().ok_or("No report directory")?)?;
    file.write_all(json.as_bytes())?;
    file.persist(path)?;
    Ok(())
}

/// Load the most recent purge result, if any
pub fn load_last() -> Option<PurgeData> {
    let path = result_path();
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}
