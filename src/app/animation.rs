//! Animation state for UI elements

use std::time::Instant;

use crate::branding::LOGO_HEIGHT;

/// Logo fade-in animation state
pub struct LogoAnim {
    start: Instant,
    pub visible_rows: usize,
}

impl LogoAnim {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            visible_rows: 0,
        }
    }

    /// Advance animation by one tick (call every frame)
    pub fn tick(&mut self) {
        // Reveal one row every 50ms
        let elapsed = self.start.elapsed().as_millis() as usize;
        self.visible_rows = (elapsed / 50).min(LOGO_HEIGHT as usize);
    }

    /// Reset animation to beginning
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.visible_rows = 0;
    }
}

impl Default for LogoAnim {
    fn default() -> Self {
        Self::new()
    }
}
