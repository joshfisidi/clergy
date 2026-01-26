use std::process::{Command, Stdio};
use std::io::Write;
use std::fmt;
use tempfile::NamedTempFile;

use crate::model::PurgeData;

/// Errors that can occur while invoking the shell backend
#[derive(Debug)]
pub enum ShellError {
    ScriptWrite(std::io::Error),
    Execution(std::io::Error),
    NonZeroExit(i32, String),
    InvalidUtf8(std::string::FromUtf8Error),
    JsonParse(serde_json::Error),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShellError::ScriptWrite(e) =>
                write!(f, "failed to write embedded shell script: {}", e),
            ShellError::Execution(e) =>
                write!(f, "failed to execute shell backend: {}", e),
            ShellError::NonZeroExit(code, stderr) =>
                write!(f, "shell backend exited with code {}: {}", code, stderr),
            ShellError::InvalidUtf8(e) =>
                write!(f, "shell backend returned invalid UTF-8: {}", e),
            ShellError::JsonParse(e) =>
                write!(f, "failed to parse backend JSON: {}", e),
        }
    }
}

impl std::error::Error for ShellError {}

/// Runs the embedded clergy purge backend and returns structured data.
///
/// This function:
/// - Writes the embedded zsh backend to a secure temp file
/// - Executes it with `--json`
/// - Captures stdout only
/// - Parses and returns `PurgeData`
///
/// It NEVER:
/// - interpolates user input
/// - runs sudo itself
/// - prints to stdout
pub fn run_purge() -> Result<PurgeData, ShellError> {
    // Embed the shell backend at compile time
    // This ensures:
    // - no runtime file dependency
    // - no tampering
    // - clean uninstall
    let script = include_str!("../backend/purge.zsh");

    // Write script to a secure temporary file
    let mut file = NamedTempFile::new()
        .map_err(ShellError::ScriptWrite)?;

    file.write_all(script.as_bytes())
        .map_err(ShellError::ScriptWrite)?;

    // Execute the backend with zsh in JSON mode
    let output = Command::new("zsh")
        .arg(file.path())
        .arg("--json")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ShellError::Execution)?
        .wait_with_output()
        .map_err(ShellError::Execution)?;

    // Ensure successful exit
    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ShellError::NonZeroExit(code, stderr));
    }

    // Decode stdout (JSON)
    let stdout = String::from_utf8(output.stdout)
        .map_err(ShellError::InvalidUtf8)?;

    // Deserialize into our frozen schema
    let data = serde_json::from_str::<PurgeData>(&stdout)
        .map_err(ShellError::JsonParse)?;

    Ok(data)
}
