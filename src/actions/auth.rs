use std::io::{self, IsTerminal};
use std::process::{Command, Stdio};

/// Call only while the terminal is in normal (cooked) mode and no UI is reading keys.
/// sudo owns the password prompt; CLERGY never reads or stores the password.
pub fn authenticate() -> io::Result<()> {
    let mut command = Command::new("sudo");
    if !io::stdin().is_terminal() {
        // Scripts must fail promptly rather than opening a hidden /dev/tty prompt.
        command.arg("-n");
    }
    let status = command
        .arg("-v")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Administrator authentication failed or was cancelled. No purge was started.",
        ))
    }
}
