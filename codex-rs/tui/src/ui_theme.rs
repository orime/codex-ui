use crate::color::is_light;
use crate::terminal_palette::best_color;
use crate::terminal_palette::default_bg;
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::RwLock;

const DEFAULT_UI_THEME_NAME: &str = "matrix";

static UI_THEME_OVERRIDE: OnceLock<RwLock<Option<String>>> = OnceLock::new();
static CURRENT_UI_THEME_NAME: OnceLock<RwLock<String>> = OnceLock::new();
static UI_THEME_SPECS: OnceLock<HashMap<&'static str, UiThemeSpec>> = OnceLock::new();

const UI_THEME_NAMES: &[&str] = &[
    "oc-2",
    "amoled",
    "aura",
    "ayu",
    "carbonfox",
    "catppuccin",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "cobalt2",
    "cursor",
    "dracula",
    "everforest",
    "flexoki",
    "github",
    "gruvbox",
    "kanagawa",
    "lucent-orng",
    "material",
    "matrix",
    "mercury",
    "monokai",
    "nightowl",
    "nord",
    "one-dark",
    "onedarkpro",
    "opencode",
    "orng",
    "osaka-jade",
    "palenight",
    "rosepine",
    "shadesofpurple",
    "solarized",
    "synthwave84",
    "tokyonight",
    "vercel",
    "vesper",
    "zenburn",
];

#[derive(Debug, Clone, Deserialize)]
struct UiThemeSpec {
    dark: UiThemeVariant,
    light: UiThemeVariant,
}

#[derive(Debug, Clone, Deserialize)]
struct UiThemeVariant {
    palette: UiThemeSeeds,
    #[serde(default)]
    overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UiThemeSeeds {
    neutral: String,
    #[serde(default)]
    ink: Option<String>,
    primary: String,
    #[serde(default)]
    accent: Option<String>,
    success: String,
    warning: String,
    error: String,
    info: String,
    #[serde(default)]
    interactive: Option<String>,
    #[serde(default, rename = "diffAdd")]
    diff_add: Option<String>,
    #[serde(default, rename = "diffDelete")]
    diff_delete: Option<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiPalette {
    pub background: Color,
    pub background_secondary: Color,
    pub background_deeper: Color,
    pub code_block_background: Color,
    pub inline_code_background: Color,
    pub user_message_background: Color,
    pub proposed_plan_background: Color,
    pub commentary_text: Color,
    pub text: Color,
    pub markdown_text: Color,
    pub text_muted: Color,
    pub text_emphasis: Color,
    pub markdown_heading: Color,
    pub markdown_link: Color,
    pub markdown_link_text: Color,
    pub markdown_code: Color,
    pub markdown_blockquote: Color,
    pub markdown_emphasis: Color,
    pub markdown_strong: Color,
    pub markdown_horizontal_rule: Color,
    pub markdown_list_item: Color,
    pub markdown_list_enumeration: Color,
    pub syntax_comment: Color,
    pub syntax_keyword: Color,
    pub syntax_function: Color,
    pub syntax_string: Color,
    pub syntax_number: Color,
    pub syntax_primitive: Color,
    pub syntax_variable: Color,
    pub syntax_property: Color,
    pub syntax_type: Color,
    pub syntax_constant: Color,
    pub syntax_operator: Color,
    pub syntax_punctuation: Color,
    pub syntax_object: Color,
    pub code_block_text: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub diff_add: Color,
    pub diff_delete: Color,
}

type Rgb = (u8, u8, u8);

#[derive(Clone, Copy)]
pub(crate) struct UiSyntaxThemePalette {
    pub background: Rgb,
    pub background_secondary: Rgb,
    pub background_deeper: Rgb,
    pub code_block_background: Rgb,
    pub text: Rgb,
    pub text_muted: Rgb,
    pub accent: Rgb,
    pub success: Rgb,
    pub error: Rgb,
    pub diff_add: Rgb,
    pub diff_delete: Rgb,
    pub diff_context: Rgb,
    pub diff_add_foreground: Rgb,
    pub diff_delete_foreground: Rgb,
    pub syntax_comment: Rgb,
    pub syntax_keyword: Rgb,
    pub syntax_function: Rgb,
    pub syntax_string: Rgb,
    pub syntax_number: Rgb,
    pub syntax_primitive: Rgb,
    pub syntax_variable: Rgb,
    pub syntax_property: Rgb,
    pub syntax_type: Rgb,
    pub syntax_constant: Rgb,
    pub syntax_operator: Rgb,
    pub syntax_punctuation: Rgb,
    pub syntax_object: Rgb,
    pub code_block_text: Rgb,
}

pub(crate) fn set_theme_override(name: Option<String>) -> Option<String> {
    let warning = validate_theme_name(name.as_deref());
    let resolved = normalize_theme_name(name.as_deref())
        .unwrap_or(DEFAULT_UI_THEME_NAME)
        .to_string();
    set_configured_theme_name(name.as_deref());
    set_runtime_theme_name(&resolved);
    warning
}

pub(crate) fn set_configured_theme_name(name: Option<&str>) -> bool {
    let normalized = normalize_theme_name(name).map(str::to_string);
    let mut guard = match configured_theme_lock().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let changed = *guard != normalized;
    *guard = normalized;
    changed
}

pub(crate) fn validate_theme_name(name: Option<&str>) -> Option<String> {
    let name = name?;
    if normalize_theme_name(Some(name)).is_some() {
        return None;
    }
    Some(format!(
        "UI theme \"{name}\" not found. Using the default UI theme \"{DEFAULT_UI_THEME_NAME}\"."
    ))
}

pub(crate) fn configured_theme_name() -> String {
    configured_theme_lock()
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_else(|poisoned| poisoned.into_inner().clone())
        .filter(|name| is_known_theme(name))
        .unwrap_or_else(|| DEFAULT_UI_THEME_NAME.to_string())
}

pub(crate) fn current_theme_name() -> String {
    let guard = match current_theme_lock().read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clone()
}

pub(crate) fn set_runtime_theme_name(name: &str) -> bool {
    let Some(name) = normalize_theme_name(Some(name)) else {
        return false;
    };
    let mut guard = match current_theme_lock().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *guard = name.to_string();
    true
}

pub(crate) fn restore_runtime_theme_from_config() {
    let configured = configured_theme_name();
    let _ = set_runtime_theme_name(&configured);
}

pub(crate) fn list_available_themes() -> Vec<String> {
    UI_THEME_NAMES.iter().map(|name| (*name).to_string()).collect()
}

pub(crate) fn active_palette() -> UiPalette {
    palette_for_terminal_bg(default_bg())
}

pub(crate) fn palette_for_terminal_bg(terminal_bg: Option<(u8, u8, u8)>) -> UiPalette {
    let theme_name = current_theme_name();
    palette_for_theme_name(theme_name.as_str(), terminal_bg)
        .unwrap_or_else(|| palette_for_theme_name(DEFAULT_UI_THEME_NAME, terminal_bg).expect("default ui theme must exist"))
}

pub(crate) fn palette_for_theme_name(
    name: &str,
    terminal_bg: Option<(u8, u8, u8)>,
) -> Option<UiPalette> {
    let theme_name = normalize_theme_name(Some(name))?;
    Some(build_palette_for_theme_name(theme_name, terminal_bg))
}

pub(crate) fn syntax_theme_alias(name: &str) -> Option<String> {
    normalize_theme_name(Some(name)).map(|name| format!("ui:{name}"))
}

pub(crate) fn resolve_syntax_theme_alias(name: &str) -> Option<&'static str> {
    name.strip_prefix("ui:")
        .and_then(|name| normalize_theme_name(Some(name)))
}

pub(crate) fn syntax_theme_palette_for_theme_name(
    name: &str,
    terminal_bg: Option<(u8, u8, u8)>,
) -> Option<UiSyntaxThemePalette> {
    let theme_name = normalize_theme_name(Some(name))?;
    Some(build_syntax_theme_palette_for_theme_name(theme_name, terminal_bg))
}

fn build_syntax_theme_palette_for_theme_name(
    theme_name: &str,
    terminal_bg: Option<(u8, u8, u8)>,
) -> UiSyntaxThemePalette {
    let spec = theme_specs()
        .get(theme_name)
        .unwrap_or_else(|| theme_specs().get(DEFAULT_UI_THEME_NAME).expect("default ui theme must exist"));
    let use_light_variant = terminal_bg.is_some_and(is_light);
    let variant = if use_light_variant {
        &spec.light
    } else {
        &spec.dark
    };
    let dark = !use_light_variant;

    let background_rgb = variant
        .override_rgb("background-base", None)
        .unwrap_or_else(|| variant.palette.neutral_rgb(None).unwrap_or((10, 14, 10)));
    let primary_rgb = variant
        .palette
        .primary_rgb(Some(background_rgb))
        .unwrap_or((46, 255, 106));
    let accent_rgb = variant
        .palette
        .accent_rgb(Some(background_rgb))
        .unwrap_or((199, 112, 255));
    let success_rgb = variant
        .palette
        .success_rgb(Some(background_rgb))
        .unwrap_or((98, 255, 148));
    let warning_rgb = variant
        .palette
        .warning_rgb(Some(background_rgb))
        .unwrap_or((230, 255, 87));
    let error_rgb = variant
        .palette
        .error_rgb(Some(background_rgb))
        .unwrap_or((255, 75, 75));
    let info_rgb = variant
        .palette
        .info_rgb(Some(background_rgb))
        .unwrap_or((48, 179, 255));
    let secondary_rgb = variant
        .palette
        .interactive_rgb(Some(background_rgb))
        .or_else(|| variant.palette.info_rgb(Some(background_rgb)))
        .unwrap_or(info_rgb);
    let text_rgb = variant
        .override_rgb("markdown-text", Some(background_rgb))
        .or_else(|| variant.override_rgb("text-base", Some(background_rgb)))
        .or_else(|| variant.palette.ink_rgb(Some(background_rgb)))
        .unwrap_or(primary_rgb);
    let code_block_background_rgb = variant
        .override_rgb("background-panel", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, primary_rgb, if dark { 0.06 } else { 0.035 }));
    let text_muted_rgb = variant
        .override_rgb("text-weak", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, text_rgb, if dark { 0.48 } else { 0.56 }));
    let syntax_comment_rgb = variant
        .override_rgb("syntax-comment", Some(background_rgb))
        .unwrap_or(text_muted_rgb);
    let syntax_keyword_rgb = variant
        .override_rgb("syntax-keyword", Some(background_rgb))
        .unwrap_or(accent_rgb);
    let syntax_function_rgb = variant
        .override_rgb("syntax-function", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_string_rgb = variant
        .override_rgb("syntax-string", Some(background_rgb))
        .unwrap_or(primary_rgb);
    let syntax_number_rgb = variant
        .override_rgb("syntax-number", Some(background_rgb))
        .unwrap_or(error_rgb);
    let syntax_primitive_rgb = variant
        .override_rgb("syntax-primitive", Some(background_rgb))
        .unwrap_or(syntax_function_rgb);
    let syntax_variable_rgb = variant
        .override_rgb("syntax-variable", Some(background_rgb))
        .unwrap_or(text_rgb);
    let syntax_property_rgb = variant
        .override_rgb("syntax-property", Some(background_rgb))
        .unwrap_or(secondary_rgb);
    let syntax_type_rgb = variant
        .override_rgb("syntax-type", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_constant_rgb = variant
        .override_rgb("syntax-constant", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_operator_rgb = variant
        .override_rgb("syntax-operator", Some(background_rgb))
        .unwrap_or(secondary_rgb);
    let syntax_punctuation_rgb = variant
        .override_rgb("syntax-punctuation", Some(background_rgb))
        .unwrap_or(text_rgb);
    let syntax_object_rgb = variant
        .override_rgb("syntax-object", Some(background_rgb))
        .unwrap_or(text_rgb);
    let code_block_text_rgb = variant
        .override_rgb("markdown-code-block", Some(background_rgb))
        .unwrap_or(text_rgb);
    let safe_code_block_text_rgb =
        ensure_contrast(code_block_text_rgb, code_block_background_rgb, text_rgb, 4.8);
    let safe_text_rgb = ensure_contrast(text_rgb, code_block_background_rgb, safe_code_block_text_rgb, 4.8);
    let background_secondary_rgb =
        overlay(code_block_background_rgb, safe_text_rgb, if dark { 0.08 } else { 0.05 });
    let background_deeper_rgb =
        overlay(code_block_background_rgb, safe_text_rgb, if dark { 0.14 } else { 0.09 });
    let diff_add_rgb = variant
        .override_rgb("diff-added-bg", Some(code_block_background_rgb))
        .or_else(|| variant.palette.diff_add_rgb(Some(background_rgb)))
        .unwrap_or(success_rgb);
    let diff_delete_rgb = variant
        .override_rgb("diff-removed-bg", Some(code_block_background_rgb))
        .or_else(|| variant.palette.diff_delete_rgb(Some(background_rgb)))
        .unwrap_or(error_rgb);
    let diff_context_rgb = variant
        .override_rgb("diff-context-bg", Some(code_block_background_rgb))
        .unwrap_or(background_secondary_rgb);
    let safe_diff_add_rgb =
        ensure_min_surface_delta(diff_add_rgb, code_block_background_rgb, success_rgb, dark);
    let safe_diff_delete_rgb =
        ensure_min_surface_delta(diff_delete_rgb, code_block_background_rgb, error_rgb, dark);
    let safe_diff_add_foreground =
        ensure_contrast(success_rgb, safe_diff_add_rgb, safe_code_block_text_rgb, 4.0);
    let safe_diff_delete_foreground =
        ensure_contrast(error_rgb, safe_diff_delete_rgb, safe_code_block_text_rgb, 4.0);

    let safe_syntax_comment_rgb =
        ensure_contrast(syntax_comment_rgb, code_block_background_rgb, safe_text_rgb, 2.6);
    let safe_syntax_keyword_rgb =
        ensure_contrast(syntax_keyword_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_function_rgb =
        ensure_contrast(syntax_function_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_string_rgb =
        ensure_contrast(syntax_string_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_number_rgb =
        ensure_contrast(syntax_number_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_primitive_rgb =
        ensure_contrast(syntax_primitive_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_variable_rgb =
        ensure_contrast(syntax_variable_rgb, code_block_background_rgb, safe_text_rgb, 4.5);
    let safe_syntax_property_rgb =
        ensure_contrast(syntax_property_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_type_rgb =
        ensure_contrast(syntax_type_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_constant_rgb =
        ensure_contrast(syntax_constant_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_operator_rgb =
        ensure_contrast(syntax_operator_rgb, code_block_background_rgb, safe_text_rgb, 3.0);
    let safe_syntax_punctuation_rgb =
        ensure_contrast(syntax_punctuation_rgb, code_block_background_rgb, safe_text_rgb, 3.2);
    let safe_syntax_object_rgb =
        ensure_contrast(syntax_object_rgb, code_block_background_rgb, safe_text_rgb, 3.2);

    UiSyntaxThemePalette {
        background: code_block_background_rgb,
        background_secondary: background_secondary_rgb,
        background_deeper: background_deeper_rgb,
        code_block_background: code_block_background_rgb,
        text: safe_text_rgb,
        text_muted: safe_syntax_comment_rgb,
        accent: accent_rgb,
        success: success_rgb,
        error: error_rgb,
        diff_add: safe_diff_add_rgb,
        diff_delete: safe_diff_delete_rgb,
        diff_context: diff_context_rgb,
        diff_add_foreground: safe_diff_add_foreground,
        diff_delete_foreground: safe_diff_delete_foreground,
        syntax_comment: safe_syntax_comment_rgb,
        syntax_keyword: safe_syntax_keyword_rgb,
        syntax_function: safe_syntax_function_rgb,
        syntax_string: safe_syntax_string_rgb,
        syntax_number: safe_syntax_number_rgb,
        syntax_primitive: safe_syntax_primitive_rgb,
        syntax_variable: safe_syntax_variable_rgb,
        syntax_property: safe_syntax_property_rgb,
        syntax_type: safe_syntax_type_rgb,
        syntax_constant: safe_syntax_constant_rgb,
        syntax_operator: safe_syntax_operator_rgb,
        syntax_punctuation: safe_syntax_punctuation_rgb,
        syntax_object: safe_syntax_object_rgb,
        code_block_text: safe_code_block_text_rgb,
    }
}

fn build_palette_for_theme_name(theme_name: &str, terminal_bg: Option<(u8, u8, u8)>) -> UiPalette {
    let spec = theme_specs()
        .get(theme_name)
        .unwrap_or_else(|| theme_specs().get(DEFAULT_UI_THEME_NAME).expect("default ui theme must exist"));
    let use_light_variant = terminal_bg.is_some_and(is_light);
    let variant = if use_light_variant {
        &spec.light
    } else {
        &spec.dark
    };
    let dark = !use_light_variant;

    let background_rgb = variant
        .override_rgb("background-base", None)
        .unwrap_or_else(|| variant.palette.neutral_rgb(None).unwrap_or((10, 14, 10)));
    let primary_rgb = variant
        .palette
        .primary_rgb(Some(background_rgb))
        .unwrap_or((46, 255, 106));
    let accent_rgb = variant
        .palette
        .accent_rgb(Some(background_rgb))
        .unwrap_or((199, 112, 255));
    let success_rgb = variant
        .palette
        .success_rgb(Some(background_rgb))
        .unwrap_or((98, 255, 148));
    let warning_rgb = variant
        .palette
        .warning_rgb(Some(background_rgb))
        .unwrap_or((230, 255, 87));
    let error_rgb = variant
        .palette
        .error_rgb(Some(background_rgb))
        .unwrap_or((255, 75, 75));
    let info_rgb = variant
        .palette
        .info_rgb(Some(background_rgb))
        .unwrap_or((48, 179, 255));
    let secondary_rgb = variant
        .palette
        .interactive_rgb(Some(background_rgb))
        .or_else(|| variant.palette.info_rgb(Some(background_rgb)))
        .unwrap_or(info_rgb);
    let text_rgb = variant
        .override_rgb("markdown-text", Some(background_rgb))
        .or_else(|| variant.override_rgb("text-base", Some(background_rgb)))
        .or_else(|| variant.palette.ink_rgb(Some(background_rgb)))
        .unwrap_or(primary_rgb);
    let text_muted_rgb = variant
        .override_rgb("text-weak", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, text_rgb, if dark { 0.48 } else { 0.56 }));
    let syntax_comment_rgb = variant
        .override_rgb("syntax-comment", Some(background_rgb))
        .unwrap_or(text_muted_rgb);
    let syntax_keyword_rgb = variant
        .override_rgb("syntax-keyword", Some(background_rgb))
        .unwrap_or(accent_rgb);
    let syntax_function_rgb = variant
        .override_rgb("syntax-function", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_string_rgb = variant
        .override_rgb("syntax-string", Some(background_rgb))
        .unwrap_or(primary_rgb);
    let syntax_number_rgb = variant
        .override_rgb("syntax-number", Some(background_rgb))
        .unwrap_or(error_rgb);
    let syntax_primitive_rgb = variant
        .override_rgb("syntax-primitive", Some(background_rgb))
        .unwrap_or(syntax_function_rgb);
    let syntax_variable_rgb = variant
        .override_rgb("syntax-variable", Some(background_rgb))
        .unwrap_or(text_rgb);
    let syntax_property_rgb = variant
        .override_rgb("syntax-property", Some(background_rgb))
        .unwrap_or(secondary_rgb);
    let syntax_type_rgb = variant
        .override_rgb("syntax-type", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_constant_rgb = variant
        .override_rgb("syntax-constant", Some(background_rgb))
        .unwrap_or(warning_rgb);
    let syntax_operator_rgb = variant
        .override_rgb("syntax-operator", Some(background_rgb))
        .unwrap_or(secondary_rgb);
    let syntax_punctuation_rgb = variant
        .override_rgb("syntax-punctuation", Some(background_rgb))
        .unwrap_or(text_rgb);
    let syntax_object_rgb = variant
        .override_rgb("syntax-object", Some(background_rgb))
        .unwrap_or(text_rgb);
    let code_block_text_rgb = variant
        .override_rgb("markdown-code-block", Some(background_rgb))
        .unwrap_or(text_rgb);

    let background_secondary_rgb = variant
        .override_rgb("background-panel", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, text_rgb, if dark { 0.08 } else { 0.05 }));
    let background_deeper_rgb = variant
        .override_rgb("background-element", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, text_rgb, if dark { 0.14 } else { 0.09 }));
    let code_block_background_rgb = background_secondary_rgb;
    let markdown_code_rgb = variant
        .override_rgb("markdown-code", Some(background_rgb))
        .unwrap_or(primary_rgb);
    let inline_code_background_rgb =
        overlay(background_rgb, markdown_code_rgb, if dark { 0.10 } else { 0.06 });
    let border_rgb = variant
        .override_rgb("border", Some(background_rgb))
        .unwrap_or_else(|| overlay(background_rgb, text_rgb, if dark { 0.18 } else { 0.20 }));
    let user_message_bg_rgb = overlay(background_secondary_rgb, secondary_rgb, if dark { 0.10 } else { 0.05 });
    let proposed_plan_bg_rgb = overlay(background_deeper_rgb, accent_rgb, if dark { 0.10 } else { 0.06 });
    let diff_add_rgb = variant
        .override_rgb("diff-added-bg", Some(code_block_background_rgb))
        .or_else(|| variant.palette.diff_add_rgb(Some(background_rgb)))
        .unwrap_or(success_rgb);
    let diff_delete_rgb = variant
        .override_rgb("diff-removed-bg", Some(code_block_background_rgb))
        .or_else(|| variant.palette.diff_delete_rgb(Some(background_rgb)))
        .unwrap_or(error_rgb);

    UiPalette {
        background: ratatui_color(background_rgb),
        background_secondary: ratatui_color(background_secondary_rgb),
        background_deeper: ratatui_color(background_deeper_rgb),
        code_block_background: ratatui_color(code_block_background_rgb),
        inline_code_background: ratatui_color(inline_code_background_rgb),
        user_message_background: ratatui_color(user_message_bg_rgb),
        proposed_plan_background: ratatui_color(proposed_plan_bg_rgb),
        commentary_text: ratatui_color(text_muted_rgb),
        text: ratatui_color(text_rgb),
        markdown_text: ratatui_color(
            variant
                .override_rgb("markdown-text", Some(background_rgb))
                .unwrap_or(text_rgb),
        ),
        text_muted: ratatui_color(text_muted_rgb),
        text_emphasis: ratatui_color(
            variant
                .override_rgb("markdown-strong", Some(background_rgb))
                .unwrap_or(warning_rgb),
        ),
        markdown_heading: ratatui_color(
            variant
                .override_rgb("markdown-heading", Some(background_rgb))
                .unwrap_or(secondary_rgb),
        ),
        markdown_link: ratatui_color(
            variant
                .override_rgb("markdown-link", Some(background_rgb))
                .unwrap_or(info_rgb),
        ),
        markdown_link_text: ratatui_color(
            variant
                .override_rgb("markdown-link-text", Some(background_rgb))
                .unwrap_or(secondary_rgb),
        ),
        markdown_code: ratatui_color(markdown_code_rgb),
        markdown_blockquote: ratatui_color(
            variant
                .override_rgb("markdown-block-quote", Some(background_rgb))
                .unwrap_or(text_muted_rgb),
        ),
        markdown_emphasis: ratatui_color(
            variant
                .override_rgb("markdown-emph", Some(background_rgb))
                .unwrap_or(accent_rgb),
        ),
        markdown_strong: ratatui_color(
            variant
                .override_rgb("markdown-strong", Some(background_rgb))
                .unwrap_or(warning_rgb),
        ),
        markdown_horizontal_rule: ratatui_color(
            variant
                .override_rgb("markdown-horizontal-rule", Some(background_rgb))
                .unwrap_or(text_muted_rgb),
        ),
        markdown_list_item: ratatui_color(
            variant
                .override_rgb("markdown-list-item", Some(background_rgb))
                .unwrap_or(info_rgb),
        ),
        markdown_list_enumeration: ratatui_color(
            variant
                .override_rgb("markdown-list-enumeration", Some(background_rgb))
                .unwrap_or(secondary_rgb),
        ),
        syntax_comment: ratatui_color(syntax_comment_rgb),
        syntax_keyword: ratatui_color(syntax_keyword_rgb),
        syntax_function: ratatui_color(syntax_function_rgb),
        syntax_string: ratatui_color(syntax_string_rgb),
        syntax_number: ratatui_color(syntax_number_rgb),
        syntax_primitive: ratatui_color(syntax_primitive_rgb),
        syntax_variable: ratatui_color(syntax_variable_rgb),
        syntax_property: ratatui_color(syntax_property_rgb),
        syntax_type: ratatui_color(syntax_type_rgb),
        syntax_constant: ratatui_color(syntax_constant_rgb),
        syntax_operator: ratatui_color(syntax_operator_rgb),
        syntax_punctuation: ratatui_color(syntax_punctuation_rgb),
        syntax_object: ratatui_color(syntax_object_rgb),
        code_block_text: ratatui_color(code_block_text_rgb),
        primary: ratatui_color(primary_rgb),
        secondary: ratatui_color(secondary_rgb),
        accent: ratatui_color(accent_rgb),
        border: ratatui_color(border_rgb),
        success: ratatui_color(success_rgb),
        warning: ratatui_color(warning_rgb),
        error: ratatui_color(error_rgb),
        info: ratatui_color(info_rgb),
        diff_add: ratatui_color(diff_add_rgb),
        diff_delete: ratatui_color(diff_delete_rgb),
    }
}

fn current_theme_lock() -> &'static RwLock<String> {
    CURRENT_UI_THEME_NAME.get_or_init(|| RwLock::new(configured_theme_name()))
}

fn configured_theme_lock() -> &'static RwLock<Option<String>> {
    UI_THEME_OVERRIDE.get_or_init(|| RwLock::new(None))
}

fn theme_specs() -> &'static HashMap<&'static str, UiThemeSpec> {
    UI_THEME_SPECS.get_or_init(|| {
        UI_THEME_NAMES
            .iter()
            .map(|name| {
                (
                    *name,
                    serde_json::from_str::<UiThemeSpec>(theme_source(name))
                        .unwrap_or_else(|err| panic!("failed to parse ui theme {name}: {err}")),
                )
            })
            .collect()
    })
}

fn normalize_theme_name(name: Option<&str>) -> Option<&'static str> {
    let name = name?;
    UI_THEME_NAMES.iter().copied().find(|candidate| *candidate == name)
}

fn is_known_theme(name: &str) -> bool {
    normalize_theme_name(Some(name)).is_some()
}

fn theme_source(name: &str) -> &'static str {
    match name {
        "oc-2" => include_str!("theme_ui_assets/oc-2.json"),
        "amoled" => include_str!("theme_ui_assets/amoled.json"),
        "aura" => include_str!("theme_ui_assets/aura.json"),
        "ayu" => include_str!("theme_ui_assets/ayu.json"),
        "carbonfox" => include_str!("theme_ui_assets/carbonfox.json"),
        "catppuccin" => include_str!("theme_ui_assets/catppuccin.json"),
        "catppuccin-frappe" => include_str!("theme_ui_assets/catppuccin-frappe.json"),
        "catppuccin-macchiato" => include_str!("theme_ui_assets/catppuccin-macchiato.json"),
        "cobalt2" => include_str!("theme_ui_assets/cobalt2.json"),
        "cursor" => include_str!("theme_ui_assets/cursor.json"),
        "dracula" => include_str!("theme_ui_assets/dracula.json"),
        "everforest" => include_str!("theme_ui_assets/everforest.json"),
        "flexoki" => include_str!("theme_ui_assets/flexoki.json"),
        "github" => include_str!("theme_ui_assets/github.json"),
        "gruvbox" => include_str!("theme_ui_assets/gruvbox.json"),
        "kanagawa" => include_str!("theme_ui_assets/kanagawa.json"),
        "lucent-orng" => include_str!("theme_ui_assets/lucent-orng.json"),
        "material" => include_str!("theme_ui_assets/material.json"),
        "matrix" => include_str!("theme_ui_assets/matrix.json"),
        "mercury" => include_str!("theme_ui_assets/mercury.json"),
        "monokai" => include_str!("theme_ui_assets/monokai.json"),
        "nightowl" => include_str!("theme_ui_assets/nightowl.json"),
        "nord" => include_str!("theme_ui_assets/nord.json"),
        "one-dark" => include_str!("theme_ui_assets/one-dark.json"),
        "onedarkpro" => include_str!("theme_ui_assets/onedarkpro.json"),
        "opencode" => include_str!("theme_ui_assets/opencode.json"),
        "orng" => include_str!("theme_ui_assets/orng.json"),
        "osaka-jade" => include_str!("theme_ui_assets/osaka-jade.json"),
        "palenight" => include_str!("theme_ui_assets/palenight.json"),
        "rosepine" => include_str!("theme_ui_assets/rosepine.json"),
        "shadesofpurple" => include_str!("theme_ui_assets/shadesofpurple.json"),
        "solarized" => include_str!("theme_ui_assets/solarized.json"),
        "synthwave84" => include_str!("theme_ui_assets/synthwave84.json"),
        "tokyonight" => include_str!("theme_ui_assets/tokyonight.json"),
        "vercel" => include_str!("theme_ui_assets/vercel.json"),
        "vesper" => include_str!("theme_ui_assets/vesper.json"),
        "zenburn" => include_str!("theme_ui_assets/zenburn.json"),
        _ => include_str!("theme_ui_assets/matrix.json"),
    }
}

impl UiThemeVariant {
    fn override_rgb(&self, key: &str, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        let value = self.overrides.get(key)?;
        parse_hex_rgb(value, background)
    }
}

impl UiThemeSeeds {
    fn neutral_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.neutral, background)
    }

    fn ink_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        self.ink
            .as_deref()
            .and_then(|hex| parse_hex_rgb(hex, background))
    }

    fn primary_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.primary, background)
    }

    fn accent_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        self.accent
            .as_deref()
            .and_then(|hex| parse_hex_rgb(hex, background))
    }

    fn success_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.success, background)
    }

    fn warning_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.warning, background)
    }

    fn error_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.error, background)
    }

    fn info_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        parse_hex_rgb(&self.info, background)
    }

    fn interactive_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        self.interactive
            .as_deref()
            .and_then(|hex| parse_hex_rgb(hex, background))
    }

    fn diff_add_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        self.diff_add
            .as_deref()
            .and_then(|hex| parse_hex_rgb(hex, background))
    }

    fn diff_delete_rgb(&self, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
        self.diff_delete
            .as_deref()
            .and_then(|hex| parse_hex_rgb(hex, background))
    }
}

fn parse_hex_rgb(hex: &str, background: Option<(u8, u8, u8)>) -> Option<(u8, u8, u8)> {
    let hex = hex.trim();
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            let background = background.unwrap_or((0, 0, 0));
            Some(overlay(background, (r, g, b), f32::from(a) / 255.0))
        }
        _ => None,
    }
}

fn overlay(base: (u8, u8, u8), tint: (u8, u8, u8), alpha: f32) -> (u8, u8, u8) {
    fn channel(base: u8, tint: u8, alpha: f32) -> u8 {
        let base = f32::from(base);
        let tint = f32::from(tint);
        ((base * (1.0 - alpha)) + (tint * alpha))
            .round()
            .clamp(0.0, 255.0) as u8
    }

    (
        channel(base.0, tint.0, alpha),
        channel(base.1, tint.1, alpha),
        channel(base.2, tint.2, alpha),
    )
}

fn ratatui_color(rgb: (u8, u8, u8)) -> Color {
    best_color(rgb)
}

fn relative_luminance(rgb: Rgb) -> f32 {
    fn channel(value: u8) -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(rgb.0) + 0.7152 * channel(rgb.1) + 0.0722 * channel(rgb.2)
}

fn contrast_ratio(a: Rgb, b: Rgb) -> f32 {
    let a = relative_luminance(a);
    let b = relative_luminance(b);
    let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
    (lighter + 0.05) / (darker + 0.05)
}

fn best_text_fallback(background: Rgb) -> Rgb {
    let white = (255, 255, 255);
    let black = (12, 16, 20);
    if contrast_ratio(white, background) >= contrast_ratio(black, background) {
        white
    } else {
        black
    }
}

fn ensure_contrast(candidate: Rgb, background: Rgb, fallback: Rgb, min_ratio: f32) -> Rgb {
    if contrast_ratio(candidate, background) >= min_ratio {
        return candidate;
    }
    if contrast_ratio(fallback, background) >= min_ratio {
        return fallback;
    }
    best_text_fallback(background)
}

fn softened_diff_background(base: Rgb, tint: Rgb, preferred_foreground: Rgb, dark: bool) -> Rgb {
    let mut alpha = if dark { 0.18 } else { 0.12 };
    let min_foreground_ratio = 3.8;
    let min_surface_delta = 1.06;

    loop {
        let candidate = overlay(base, tint, alpha);
        if contrast_ratio(preferred_foreground, candidate) >= min_foreground_ratio
            && contrast_ratio(candidate, base) >= min_surface_delta
        {
            return candidate;
        }
        if alpha <= 0.04 {
            return overlay(base, tint, 0.08);
        }
        alpha -= 0.02;
    }
}

fn ensure_min_surface_delta(candidate: Rgb, base: Rgb, preferred_foreground: Rgb, dark: bool) -> Rgb {
    let min_surface_delta = 1.06;
    if contrast_ratio(candidate, base) >= min_surface_delta
        && contrast_ratio(preferred_foreground, candidate) >= 3.8
    {
        return candidate;
    }
    softened_diff_background(base, candidate, preferred_foreground, dark)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_theme_palette_keeps_cobalt2_code_block_readable() {
        let palette =
            build_syntax_theme_palette_for_theme_name("cobalt2", Some((0x19, 0x35, 0x49)));
        assert!(contrast_ratio(palette.code_block_text, palette.code_block_background) >= 4.8);
        assert!(contrast_ratio(palette.diff_add_foreground, palette.diff_add) >= 3.8);
        assert!(contrast_ratio(palette.diff_delete_foreground, palette.diff_delete) >= 3.8);
    }

    #[test]
    fn syntax_theme_palette_uses_muted_diff_backgrounds() {
        let palette = build_syntax_theme_palette_for_theme_name("matrix", None);
        assert!(contrast_ratio(palette.diff_add, palette.code_block_background) < 1.8);
        assert!(contrast_ratio(palette.diff_delete, palette.code_block_background) < 1.8);
        assert!(contrast_ratio(palette.diff_add, palette.code_block_background) > 1.05);
        assert!(contrast_ratio(palette.diff_delete, palette.code_block_background) > 1.05);
    }
}
