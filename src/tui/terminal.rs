use std::io::{self, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use ratatui::DefaultTerminal;
use signal_hook::{consts::SIGINT, SigId};

/// Restore cooked input, the main screen and the cursor on every exit path.
pub(super) struct TerminalSession {
    pub terminal: DefaultTerminal,
    interrupted: Arc<AtomicBool>,
    sigint: SigId,
}

impl TerminalSession {
    pub fn new() -> io::Result<Self> {
        // During sudo, Ctrl+C reaches the whole foreground process group. Catch
        // it here so sudo can cancel its prompt without killing the UI parent.
        // exec resets this caught handler in the child process.
        let interrupted = Arc::new(AtomicBool::new(false));
        let sigint = signal_hook::flag::register(SIGINT, Arc::clone(&interrupted))?;
        // Ratatui also installs a hook to restore the terminal before panic output.
        match ratatui::try_init() {
            Ok(terminal) => Ok(Self {
                terminal,
                interrupted,
                sigint,
            }),
            Err(error) => {
                let _ = ratatui::try_restore();
                signal_hook::low_level::unregister(sigint);
                Err(error)
            }
        }
    }

    pub fn interrupted(&self) -> bool {
        self.interrupted.swap(false, Ordering::Relaxed)
    }

    pub fn authenticate(&mut self) -> io::Result<io::Result<()>> {
        ratatui::try_restore()?;
        self.terminal.show_cursor()?;
        eprintln!("CLERGY · Confirmed purge. Authenticate with sudo (password input is hidden).");
        // No event polling or drawing occurs until sudo exits, including failures.
        let result = crate::actions::authenticate();
        self.interrupted.store(false, Ordering::Relaxed);

        enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        // The alternate screen and Ratatui's diff buffer must both be reset.
        self.terminal.clear()?;
        self.terminal.backend_mut().flush()?;
        Ok(result)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = ratatui::try_restore();
        let _ = self.terminal.show_cursor();
        signal_hook::low_level::unregister(self.sigint);
    }
}
