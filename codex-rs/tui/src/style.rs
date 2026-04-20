use crate::terminal_palette::default_bg;
use crate::ui_theme;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;

pub fn user_message_style() -> Style {
    user_message_style_for(default_bg())
}

pub fn proposed_plan_style() -> Style {
    proposed_plan_style_for(default_bg())
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
