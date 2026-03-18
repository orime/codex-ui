//! Shared popup-related constants for bottom pane widgets.

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::key_hint;
use crate::style::opencode_secondary;
use crate::style::opencode_text_muted;

/// Maximum number of rows any popup should attempt to display.
/// Keep this consistent across all popups for a uniform feel.
pub(crate) const MAX_POPUP_ROWS: usize = 8;

/// Standard footer hint text used by popups.
pub(crate) fn standard_popup_hint_line() -> Line<'static> {
    Line::from(vec![
        "Press ".fg(opencode_text_muted()),
        key_hint::plain(KeyCode::Enter).into(),
        " to confirm".fg(opencode_secondary()),
        " or ".fg(opencode_text_muted()),
        key_hint::plain(KeyCode::Esc).into(),
        " to go back".fg(opencode_secondary()),
    ])
}
