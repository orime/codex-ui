use crate::color::is_light;
use crate::terminal_palette::best_color;
use crate::terminal_palette::default_bg;
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
    match terminal_bg {
        Some(bg) => Style::default().fg(opencode_text()).bg(user_message_bg(bg)),
        None => Style::default()
            .fg(opencode_text())
            .bg(opencode_background_secondary()),
    }
}

pub fn proposed_plan_style_for(terminal_bg: Option<(u8, u8, u8)>) -> Style {
    match terminal_bg {
        Some(bg) => Style::default()
            .fg(opencode_text())
            .bg(proposed_plan_bg(bg)),
        None => Style::default()
            .fg(opencode_text())
            .bg(opencode_background_deeper()),
    }
}

#[allow(clippy::disallowed_methods)]
pub fn user_message_bg(terminal_bg: (u8, u8, u8)) -> Color {
    if is_light(terminal_bg) {
        best_color((228, 235, 225))
    } else {
        best_color((14, 19, 13))
    }
}

#[allow(clippy::disallowed_methods)]
pub fn proposed_plan_bg(terminal_bg: (u8, u8, u8)) -> Color {
    if is_light(terminal_bg) {
        best_color((218, 225, 215))
    } else {
        best_color((20, 28, 18))
    }
}

fn palette_color(dark: (u8, u8, u8), light: (u8, u8, u8)) -> Color {
    let target = if default_bg().is_some_and(is_light) {
        light
    } else {
        dark
    };
    best_color(target)
}

pub fn opencode_background() -> Color {
    palette_color((10, 14, 10), (238, 243, 234))
}

pub fn opencode_background_secondary() -> Color {
    palette_color((14, 19, 13), (228, 235, 225))
}

pub fn opencode_background_deeper() -> Color {
    palette_color((20, 28, 18), (218, 225, 215))
}

pub fn opencode_code_block_background() -> Color {
    palette_color((14, 19, 13), (228, 235, 225))
}

pub fn opencode_inline_code_background() -> Color {
    palette_color((10, 14, 10), (238, 243, 234))
}

pub fn opencode_commentary_text() -> Color {
    palette_color((140, 163, 145), (116, 132, 118))
}

pub fn opencode_text() -> Color {
    palette_color((98, 255, 148), (32, 48, 34))
}

pub fn opencode_markdown_text() -> Color {
    palette_color((98, 255, 148), (32, 48, 34))
}

pub fn opencode_text_muted() -> Color {
    palette_color((140, 163, 145), (116, 132, 118))
}

pub fn opencode_text_emphasis() -> Color {
    palette_color((230, 255, 87), (255, 168, 61))
}

pub fn opencode_markdown_heading() -> Color {
    palette_color((0, 239, 255), (36, 246, 217))
}

pub fn opencode_markdown_link() -> Color {
    palette_color((48, 179, 255), (48, 179, 255))
}

pub fn opencode_markdown_link_text() -> Color {
    palette_color((36, 246, 217), (36, 246, 217))
}

pub fn opencode_markdown_code() -> Color {
    palette_color((28, 194, 75), (28, 194, 75))
}

pub fn opencode_markdown_blockquote() -> Color {
    palette_color((140, 163, 145), (116, 132, 118))
}

pub fn opencode_markdown_emphasis() -> Color {
    palette_color((255, 168, 61), (255, 168, 61))
}

pub fn opencode_markdown_strong() -> Color {
    palette_color((230, 255, 87), (230, 255, 87))
}

pub fn opencode_markdown_list_item() -> Color {
    palette_color((48, 179, 255), (48, 179, 255))
}

pub fn opencode_markdown_list_enumeration() -> Color {
    palette_color((36, 246, 217), (36, 246, 217))
}

pub fn opencode_primary() -> Color {
    palette_color((46, 255, 106), (28, 194, 75))
}

pub fn opencode_secondary() -> Color {
    palette_color((0, 239, 255), (36, 246, 217))
}

pub fn opencode_accent() -> Color {
    palette_color((199, 112, 255), (199, 112, 255))
}

pub fn opencode_border() -> Color {
    palette_color((30, 42, 27), (116, 132, 118))
}

pub fn opencode_success() -> Color {
    palette_color((98, 255, 148), (28, 194, 75))
}

pub fn opencode_warning() -> Color {
    palette_color((230, 255, 87), (230, 255, 87))
}

#[allow(dead_code)]
pub fn opencode_error() -> Color {
    palette_color((255, 75, 75), (255, 75, 75))
}

pub fn opencode_info() -> Color {
    palette_color((48, 179, 255), (48, 179, 255))
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
