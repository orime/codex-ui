use crate::color::blend;
use crate::color::is_light;
use crate::terminal_palette::StdoutColorLevel;
use crate::terminal_palette::best_color;
use crate::terminal_palette::default_bg;
use crate::terminal_palette::default_fg;
use crate::terminal_palette::rgb_color;
use crate::terminal_palette::stdout_color_level;
use crate::ui_theme;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;

const LIGHT_BG_ACCENT_RGB: (u8, u8, u8) = (0, 95, 135);
// Decorative table rules should remain visible without competing with cell content.
const TABLE_SEPARATOR_FG_ALPHA: f32 = 0.20;

pub fn user_message_style() -> Style {
    user_message_style_for(default_bg())
}

pub fn proposed_plan_style() -> Style {
    proposed_plan_style_for(default_bg())
}

/// Returns a low-contrast rule style for separators within markdown tables.
pub(crate) fn table_separator_style() -> Style {
    table_separator_style_for(default_fg(), default_bg(), stdout_color_level())
}

/// Returns the shared accent style for active or selected TUI controls.
pub(crate) fn accent_style() -> Style {
    accent_style_for(default_bg())
}

/// Returns the style for a user-authored message using the provided terminal background.
pub fn user_message_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    let palette = ui_theme::palette_for_terminal_bg(terminal_bg.or_else(default_bg));
    Style::default()
        .fg(palette.text)
        .bg(palette.user_message_background)
}

pub fn proposed_plan_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    let palette = ui_theme::palette_for_terminal_bg(terminal_bg.or_else(default_bg));
    Style::default()
        .fg(palette.text)
        .bg(palette.proposed_plan_background)
}

/// Returns the shared accent style for the provided terminal background.
pub(crate) fn accent_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    if terminal_bg.is_some_and(is_light) {
        Style::default().fg(best_color(LIGHT_BG_ACCENT_RGB)).bold()
    } else {
        Style::default().fg(Color::Cyan).bold()
    }
}

fn table_separator_style_for(
    terminal_fg: Option<(u8, u8, u8)>,
    terminal_bg: Option<(u8, u8, u8)>,
    color_level: StdoutColorLevel,
) -> Style {
    let (Some(fg), Some(bg)) = (terminal_fg, terminal_bg) else {
        return Style::default().dim();
    };
    let separator_rgb = blend(fg, bg, TABLE_SEPARATOR_FG_ALPHA);
    match color_level {
        StdoutColorLevel::TrueColor => Style::default().fg(rgb_color(separator_rgb)),
        StdoutColorLevel::Ansi256 => Style::default().fg(best_color(separator_rgb)),
        StdoutColorLevel::Ansi16 | StdoutColorLevel::Unknown => Style::default().dim(),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    let (top, alpha) = if is_light(terminal_bg) {
        ((0, 0, 0), 0.04)
    } else {
        ((255, 255, 255), 0.12)
    };
    best_color(blend(top, terminal_bg, alpha))
}

#[allow(clippy::disallowed_methods)]
pub fn proposed_plan_bg(terminal_bg: (u8, u8, u8)) -> Color {
    user_message_bg(terminal_bg)
}

pub fn opencode_background() -> Color {
    ui_theme::active_palette().background
}

pub fn opencode_background_secondary() -> Color {
    ui_theme::active_palette().background_secondary
}

pub fn opencode_background_deeper() -> Color {
    ui_theme::active_palette().background_deeper
}

pub fn opencode_code_block_background() -> Color {
    ui_theme::active_palette().code_block_background
}

pub fn opencode_inline_code_background() -> Color {
    ui_theme::active_palette().inline_code_background
}

pub fn opencode_commentary_text() -> Color {
    ui_theme::active_palette().commentary_text
}

pub fn opencode_text() -> Color {
    ui_theme::active_palette().text
}

pub fn opencode_markdown_text() -> Color {
    ui_theme::active_palette().markdown_text
}

pub fn opencode_text_muted() -> Color {
    ui_theme::active_palette().text_muted
}

pub fn opencode_text_emphasis() -> Color {
    ui_theme::active_palette().text_emphasis
}

pub fn opencode_markdown_heading() -> Color {
    ui_theme::active_palette().markdown_heading
}

pub fn opencode_markdown_link() -> Color {
    ui_theme::active_palette().markdown_link
}

pub fn opencode_markdown_link_text() -> Color {
    ui_theme::active_palette().markdown_link_text
}

pub fn opencode_markdown_code() -> Color {
    ui_theme::active_palette().markdown_code
}

pub fn opencode_markdown_blockquote() -> Color {
    ui_theme::active_palette().markdown_blockquote
}

pub fn opencode_markdown_emphasis() -> Color {
    ui_theme::active_palette().markdown_emphasis
}

pub fn opencode_markdown_strong() -> Color {
    ui_theme::active_palette().markdown_strong
}

pub fn opencode_markdown_horizontal_rule() -> Color {
    ui_theme::active_palette().markdown_horizontal_rule
}

pub fn opencode_markdown_list_item() -> Color {
    ui_theme::active_palette().markdown_list_item
}

pub fn opencode_markdown_list_enumeration() -> Color {
    ui_theme::active_palette().markdown_list_enumeration
}

pub fn opencode_primary() -> Color {
    ui_theme::active_palette().primary
}

pub fn opencode_secondary() -> Color {
    ui_theme::active_palette().secondary
}

pub fn opencode_accent() -> Color {
    ui_theme::active_palette().accent
}

pub fn opencode_border() -> Color {
    ui_theme::active_palette().border
}

pub fn opencode_success() -> Color {
    ui_theme::active_palette().success
}

pub fn opencode_warning() -> Color {
    ui_theme::active_palette().warning
}

pub fn opencode_error() -> Color {
    ui_theme::active_palette().error
}

pub fn opencode_info() -> Color {
    ui_theme::active_palette().info
}

pub fn opencode_surface_style() -> Style {
    Style::default()
        .fg(opencode_text())
        .bg(opencode_background_secondary())
}

pub fn opencode_primary_style() -> Style {
    Style::default().fg(opencode_primary())
}

pub fn opencode_secondary_style() -> Style {
    Style::default().fg(opencode_secondary())
}

pub fn opencode_accent_style() -> Style {
    Style::default().fg(opencode_accent())
}

pub fn opencode_warning_style() -> Style {
    Style::default().fg(opencode_warning())
}

pub fn opencode_error_style() -> Style {
    Style::default().fg(opencode_error())
}

pub fn opencode_info_style() -> Style {
    Style::default().fg(opencode_info())
}

pub fn opencode_muted_style() -> Style {
    Style::default().fg(opencode_text_muted())
}

pub fn opencode_link_style() -> Style {
    Style::default()
        .fg(opencode_info())
        .add_modifier(Modifier::UNDERLINED)
}

pub fn opencode_selected_style() -> Style {
    Style::default()
        .fg(opencode_background())
        .bg(opencode_primary())
        .add_modifier(Modifier::BOLD)
}

pub fn opencode_key_hint_style() -> Style {
    Style::default()
        .fg(opencode_background())
        .bg(opencode_primary())
        .add_modifier(Modifier::BOLD)
}

pub fn opencode_muted_badge_style() -> Style {
    Style::default()
        .fg(opencode_text())
        .bg(opencode_background_deeper())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Modifier;

    #[test]
    fn accent_style_uses_darker_cyan_on_light_backgrounds() {
        let style = accent_style_for(Some((255, 255, 255)));

        assert_eq!(style.fg, Some(best_color(LIGHT_BG_ACCENT_RGB)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn accent_style_uses_cyan_on_dark_or_unknown_backgrounds() {
        let expected = Style::default().fg(Color::Cyan).bold();

        assert_eq!(accent_style_for(Some((0, 0, 0))), expected);
        assert_eq!(accent_style_for(/*terminal_bg*/ None), expected);
    }

    #[test]
    fn table_separator_blends_toward_dark_background() {
        let style = table_separator_style_for(
            Some((255, 255, 255)),
            Some((0, 0, 0)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((51, 51, 51))));
    }

    #[test]
    fn table_separator_blends_toward_light_background() {
        let style = table_separator_style_for(
            Some((0, 0, 0)),
            Some((255, 255, 255)),
            StdoutColorLevel::TrueColor,
        );

        assert_eq!(style.fg, Some(rgb_color((204, 204, 204))));
    }

    #[test]
    fn table_separator_dims_when_palette_aware_color_is_unavailable() {
        let expected = Style::default().dim();

        assert_eq!(
            table_separator_style_for(
                Some((255, 255, 255)),
                Some((0, 0, 0)),
                StdoutColorLevel::Ansi16,
            ),
            expected
        );
        assert_eq!(
            table_separator_style_for(
                /*terminal_fg*/ None,
                Some((0, 0, 0)),
                StdoutColorLevel::TrueColor,
            ),
            expected
        );
    }
}
