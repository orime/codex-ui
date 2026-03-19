//! Markdown rendering for the TUI transcript.
//!
//! This renderer intentionally treats local file links differently from normal web links. For
//! local paths, the displayed text comes from the destination, not the markdown label, so
//! transcripts show the real file target (including normalized location suffixes) and can shorten
//! absolute paths relative to a known working directory.

use crate::render::highlight::highlight_code_to_lines;
use crate::render::line_utils::line_to_static;
use crate::style::opencode_accent;
use crate::style::opencode_code_block_background;
use crate::style::opencode_inline_code_background;
use crate::style::opencode_markdown_blockquote;
use crate::style::opencode_markdown_emphasis;
use crate::style::opencode_markdown_heading;
use crate::style::opencode_markdown_link;
use crate::style::opencode_markdown_link_text;
use crate::style::opencode_markdown_list_enumeration;
use crate::style::opencode_markdown_list_item;
use crate::style::opencode_markdown_strong;
use crate::style::opencode_markdown_text;
use crate::style::opencode_text_muted;
use crate::style::opencode_text_emphasis;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_line;
use codex_utils_string::normalize_markdown_hash_location_suffix;
use dirs::home_dir;
use pulldown_cmark::CodeBlockKind;
use pulldown_cmark::CowStr;
use pulldown_cmark::Event;
use pulldown_cmark::HeadingLevel;
use pulldown_cmark::Options;
use pulldown_cmark::Parser;
use pulldown_cmark::Tag;
use pulldown_cmark::TagEnd;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use regex_lite::Regex;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use unicode_width::UnicodeWidthStr;
use url::Url;

struct MarkdownStyles {
    h1: Style,
    h2: Style,
    h3: Style,
    h4: Style,
    h5: Style,
    h6: Style,
    code: Style,
    emphasis: Style,
    strong: Style,
    strikethrough: Style,
    ordered_list_marker: Style,
    unordered_list_marker: Style,
    link_text: Style,
    link_destination: Style,
    blockquote: Style,
}

impl Default for MarkdownStyles {
    fn default() -> Self {
        use ratatui::style::Stylize;

        let heading = Style::default()
            .fg(opencode_markdown_heading())
            .add_modifier(ratatui::style::Modifier::BOLD);
        let heading_level_two = Style::default()
            .fg(opencode_accent())
            .add_modifier(ratatui::style::Modifier::BOLD);
        Self {
            h1: heading,
            h2: heading_level_two,
            h3: heading,
            h4: heading,
            h5: heading,
            h6: heading,
            code: Style::default()
                .fg(opencode_text_emphasis())
                .bg(opencode_inline_code_background()),
            emphasis: Style::default().fg(opencode_markdown_emphasis()).italic(),
            strong: Style::default()
                .fg(opencode_markdown_strong())
                .add_modifier(ratatui::style::Modifier::BOLD),
            strikethrough: Style::default().crossed_out(),
            ordered_list_marker: Style::default().fg(opencode_markdown_list_enumeration()),
            unordered_list_marker: Style::default().fg(opencode_markdown_list_item()),
            link_text: Style::default()
                .fg(opencode_markdown_link_text())
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
            link_destination: Style::default()
                .fg(opencode_markdown_link())
                .add_modifier(ratatui::style::Modifier::UNDERLINED),
            blockquote: Style::default().fg(opencode_markdown_blockquote()),
        }
    }
}

#[derive(Clone, Debug)]
struct IndentContext {
    prefix: Vec<Span<'static>>,
    marker: Option<Vec<Span<'static>>>,
    is_list: bool,
}

impl IndentContext {
    fn new(prefix: Vec<Span<'static>>, marker: Option<Vec<Span<'static>>>, is_list: bool) -> Self {
        Self {
            prefix,
            marker,
            is_list,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct TableState {
    rows: Vec<(bool, Vec<String>)>,
    current_row: Vec<String>,
    current_cell: String,
    in_header: bool,
    current_row_is_header: bool,
}

pub fn render_markdown_text(input: &str) -> Text<'static> {
    render_markdown_text_with_width(input, /*width*/ None)
}

/// Render markdown using the current process working directory for local file-link display.
pub(crate) fn render_markdown_text_with_width(input: &str, width: Option<usize>) -> Text<'static> {
    let cwd = std::env::current_dir().ok();
    render_markdown_text_with_width_and_cwd(input, width, cwd.as_deref())
}

/// Render markdown with an explicit working directory for local file links.
///
/// The `cwd` parameter controls how absolute local targets are shortened before display. Passing
/// the session cwd keeps full renders, history cells, and streamed deltas visually aligned even
/// when rendering happens away from the process cwd.
pub(crate) fn render_markdown_text_with_width_and_cwd(
    input: &str,
    width: Option<usize>,
    cwd: Option<&Path>,
) -> Text<'static> {
    let normalized = normalize_pipe_table_spacing(input);
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(&normalized, options);
    let mut w = Writer::new(parser, width, cwd);
    w.run();
    w.text
}

fn normalize_pipe_table_spacing(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut idx = 0usize;

    while idx < lines.len() {
        if let Some((next_idx, normalized_table)) = normalize_pipe_table_block(&lines, idx) {
            out.extend(normalized_table);
            idx = next_idx;
            continue;
        }

        if has_duplicate_table_header_prefix(&lines, idx) {
            idx += 1;
            continue;
        }

        out.push(lines[idx].to_string());
        idx += 1;
    }

    let mut normalized = out.join("\n");
    if input.ends_with('\n') {
        normalized.push('\n');
    }
    normalized
}

fn has_duplicate_table_header_prefix(lines: &[&str], start_idx: usize) -> bool {
    let Some(header_line) = lines.get(start_idx).map(|line| line.trim()) else {
        return false;
    };
    if header_line.is_empty()
        || looks_like_pipe_table_delimiter(header_line)
        || !looks_like_pipe_table_row(header_line)
    {
        return false;
    }

    let header_cells = normalize_pipe_table_cells(header_line);
    if header_cells.len() < 2 {
        return false;
    }

    let mut cursor = start_idx + 1;
    let mut saw_duplicate_header = false;

    while cursor < lines.len() {
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }

        if normalize_pipe_table_cells(trimmed) == header_cells {
            saw_duplicate_header = true;
            cursor += 1;
            continue;
        }

        return saw_duplicate_header && looks_like_pipe_table_delimiter(trimmed);
    }

    false
}

fn normalize_pipe_table_block(lines: &[&str], start_idx: usize) -> Option<(usize, Vec<String>)> {
    let header_line = lines.get(start_idx)?.trim();
    if header_line.is_empty()
        || looks_like_pipe_table_delimiter(header_line)
        || !looks_like_pipe_table_row(header_line)
    {
        return None;
    }

    let header_cells = normalize_pipe_table_cells(header_line);
    if header_cells.len() < 2 {
        return None;
    }

    let mut cursor = start_idx + 1;
    let mut delimiter_line: Option<&str> = None;

    while cursor < lines.len() {
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }

        if looks_like_pipe_table_delimiter(trimmed) {
            delimiter_line = Some(trimmed);
            cursor += 1;
            break;
        }

        if normalize_pipe_table_cells(trimmed) == header_cells {
            cursor += 1;
            continue;
        }

        return None;
    }

    let Some(delimiter_line) = delimiter_line else {
        return None;
    };

    let mut normalized = vec![
        canonicalize_pipe_table_row(header_line),
        canonicalize_pipe_table_delimiter(delimiter_line),
    ];

    while cursor < lines.len() {
        let trimmed = lines[cursor].trim();
        if trimmed.is_empty() {
            cursor += 1;
            continue;
        }

        if looks_like_pipe_table_delimiter(trimmed) {
            break;
        }

        if looks_like_pipe_table_row(trimmed) {
            normalized.push(canonicalize_pipe_table_row(trimmed));
            cursor += 1;
            continue;
        }

        break;
    }

    Some((cursor, normalized))
}

fn looks_like_pipe_table_row(line: &str) -> bool {
    split_pipe_table_cells(line).is_some()
}

fn looks_like_pipe_table_delimiter(line: &str) -> bool {
    let Some(cells) = split_pipe_table_cells(line) else {
        return false;
    };

    cells.iter().all(|cell| {
        let cell = cell.trim();
        !cell.is_empty() && cell.chars().all(|ch| matches!(ch, '-' | ':' | ' '))
    })
}

fn normalize_pipe_table_cells(line: &str) -> Vec<String> {
    split_pipe_table_cells(line)
        .unwrap_or_default()
        .into_iter()
        .map(|cell| cell.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

fn canonicalize_pipe_table_row(line: &str) -> String {
    let cells = split_pipe_table_cells(line).unwrap_or_default();
    format!(
        "| {} |",
        cells
            .into_iter()
            .map(|cell| protect_table_cell_for_parser(cell.trim()))
            .collect::<Vec<_>>()
            .join(" | ")
    )
}

fn canonicalize_pipe_table_delimiter(line: &str) -> String {
    let cells = split_pipe_table_cells(line).unwrap_or_default();
    let normalized = cells
        .into_iter()
        .map(|cell| {
            let trimmed = cell.trim();
            let left_aligned = trimmed.starts_with(':');
            let right_aligned = trimmed.ends_with(':');
            let dash_count = trimmed.chars().filter(|ch| *ch == '-').count().max(3);
            let mut normalized = "-".repeat(dash_count);
            if left_aligned {
                normalized.insert(0, ':');
            }
            if right_aligned {
                normalized.push(':');
            }
            normalized
        })
        .collect::<Vec<_>>();
    format!("| {} |", normalized.join(" | "))
}

fn split_pipe_table_cells(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.contains('|') {
        return None;
    }

    let mut core = trimmed;
    if let Some(rest) = core.strip_prefix('|') {
        core = rest;
    }
    if let Some(rest) = core.strip_suffix('|') {
        core = rest;
    }

    let mut cells = Vec::new();
    let mut cell_start = 0usize;
    let mut idx = 0usize;
    let mut code_tick_run: Option<usize> = None;

    while idx < core.len() {
        let ch = core[idx..].chars().next().unwrap();
        let ch_len = ch.len_utf8();

        if ch == '\\' {
            idx += ch_len;
            if idx < core.len() {
                let escaped = core[idx..].chars().next().unwrap();
                idx += escaped.len_utf8();
            }
            continue;
        }

        if ch == '`' {
            let tick_start = idx;
            let mut tick_count = 0usize;
            while idx < core.len() {
                let tick = core[idx..].chars().next().unwrap();
                if tick != '`' {
                    break;
                }
                tick_count += 1;
                idx += tick.len_utf8();
            }

            match code_tick_run {
                Some(open_tick_count) if open_tick_count == tick_count => code_tick_run = None,
                None => code_tick_run = Some(tick_count),
                _ => {}
            }

            if idx == tick_start {
                idx += ch_len;
            }
            continue;
        }

        if ch == '|' && code_tick_run.is_none() {
            cells.push(core[cell_start..idx].to_string());
            idx += ch_len;
            cell_start = idx;
            continue;
        }

        idx += ch_len;
    }

    cells.push(core[cell_start..].to_string());
    if cells.len() < 2 {
        return None;
    }

    Some(cells)
}

const TABLE_PIPE_SENTINEL: char = '\u{E000}';

fn protect_table_cell_for_parser(cell: &str) -> String {
    if !cell.contains('|') || !cell.contains('`') {
        return cell.to_string();
    }

    let mut out = String::with_capacity(cell.len());
    let mut idx = 0usize;
    let mut code_tick_run: Option<usize> = None;

    while idx < cell.len() {
        let ch = cell[idx..].chars().next().unwrap();
        let ch_len = ch.len_utf8();

        if ch == '\\' {
            out.push(ch);
            idx += ch_len;
            if idx < cell.len() {
                let escaped = cell[idx..].chars().next().unwrap();
                out.push(escaped);
                idx += escaped.len_utf8();
            }
            continue;
        }

        if ch == '`' {
            let tick_start = idx;
            let mut tick_count = 0usize;
            while idx < cell.len() {
                let tick = cell[idx..].chars().next().unwrap();
                if tick != '`' {
                    break;
                }
                out.push(tick);
                tick_count += 1;
                idx += tick.len_utf8();
            }

            match code_tick_run {
                Some(open_tick_count) if open_tick_count == tick_count => code_tick_run = None,
                None => code_tick_run = Some(tick_count),
                _ => {}
            }

            if idx == tick_start {
                out.push(ch);
                idx += ch_len;
            }
            continue;
        }

        if ch == '|' && code_tick_run.is_some() {
            out.push(TABLE_PIPE_SENTINEL);
        } else {
            out.push(ch);
        }
        idx += ch_len;
    }

    out
}
#[derive(Clone, Debug)]
struct LinkState {
    destination: String,
    show_destination: bool,
    /// Pre-rendered display text for local file links.
    ///
    /// When this is present, the markdown label is intentionally suppressed so the rendered
    /// transcript always reflects the real target path.
    local_target_display: Option<String>,
}

fn should_render_link_destination(dest_url: &str) -> bool {
    !is_local_path_like_link(dest_url)
}

static COLON_LOCATION_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(
        || match Regex::new(r":\d+(?::\d+)?(?:[-–]\d+(?::\d+)?)?$") {
            Ok(regex) => regex,
            Err(error) => panic!("invalid location suffix regex: {error}"),
        },
    );

// Covered by load_location_suffix_regexes.
static HASH_LOCATION_SUFFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| match Regex::new(r"^L\d+(?:C\d+)?(?:-L\d+(?:C\d+)?)?$") {
        Ok(regex) => regex,
        Err(error) => panic!("invalid hash location regex: {error}"),
    });

struct Writer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    iter: I,
    text: Text<'static>,
    styles: MarkdownStyles,
    inline_styles: Vec<Style>,
    indent_stack: Vec<IndentContext>,
    list_indices: Vec<Option<u64>>,
    link: Option<LinkState>,
    needs_newline: bool,
    pending_marker_line: bool,
    in_paragraph: bool,
    in_code_block: bool,
    code_block_lang: Option<String>,
    code_block_buffer: String,
    wrap_width: Option<usize>,
    cwd: Option<PathBuf>,
    line_ends_with_local_link_target: bool,
    pending_local_link_soft_break: bool,
    pending_list_lead_highlight: bool,
    table_state: Option<TableState>,
    in_math_block: bool,
    current_line_content: Option<Line<'static>>,
    current_initial_indent: Vec<Span<'static>>,
    current_subsequent_indent: Vec<Span<'static>>,
    current_line_style: Style,
    current_line_in_code_block: bool,
}

impl<'a, I> Writer<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I, wrap_width: Option<usize>, cwd: Option<&Path>) -> Self {
        Self {
            iter,
            text: Text::default(),
            styles: MarkdownStyles::default(),
            inline_styles: Vec::new(),
            indent_stack: Vec::new(),
            list_indices: Vec::new(),
            link: None,
            needs_newline: false,
            pending_marker_line: false,
            in_paragraph: false,
            in_code_block: false,
            code_block_lang: None,
            code_block_buffer: String::new(),
            wrap_width,
            cwd: cwd.map(Path::to_path_buf),
            line_ends_with_local_link_target: false,
            pending_local_link_soft_break: false,
            pending_list_lead_highlight: false,
            table_state: None,
            in_math_block: false,
            current_line_content: None,
            current_initial_indent: Vec::new(),
            current_subsequent_indent: Vec::new(),
            current_line_style: Style::default(),
            current_line_in_code_block: false,
        }
    }

    fn run(&mut self) {
        while let Some(ev) = self.iter.next() {
            self.handle_event(ev);
        }
        self.flush_current_line();
    }

    fn handle_event(&mut self, event: Event<'a>) {
        self.prepare_for_event(&event);
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text(text),
            Event::Code(code) => self.code(code),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Rule => {
                if self.table_state.is_some() {
                    self.push_table_text("---");
                    return;
                }
                self.flush_current_line();
                if !self.text.lines.is_empty() {
                    self.push_blank_line();
                }
                self.push_line(Line::from(vec![Span::styled(
                    "---",
                    Style::default().fg(opencode_accent()),
                )]));
                self.needs_newline = true;
            }
            Event::Html(html) => self.html(html, /*inline*/ false),
            Event::InlineHtml(html) => self.html(html, /*inline*/ true),
            Event::FootnoteReference(_) => {}
            Event::TaskListMarker(checked) => self.task_list_marker(checked),
        }
    }

    fn prepare_for_event(&mut self, event: &Event<'a>) {
        if !self.pending_local_link_soft_break {
            return;
        }

        // Local file links render from the destination at `TagEnd::Link`, so a Markdown soft break
        // immediately before a descriptive `: ...` should stay inline instead of splitting the
        // list item across two lines.
        if matches!(event, Event::Text(text) if text.trim_start().starts_with(':')) {
            self.pending_local_link_soft_break = false;
            return;
        }

        self.pending_local_link_soft_break = false;
        self.push_line(Line::default());
    }

    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => self.start_paragraph(),
            Tag::Heading { level, .. } => self.start_heading(level),
            Tag::BlockQuote => self.start_blockquote(),
            Tag::CodeBlock(kind) => {
                let indent = match kind {
                    CodeBlockKind::Fenced(_) => None,
                    CodeBlockKind::Indented => Some(Span::from(" ".repeat(4))),
                };
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    CodeBlockKind::Indented => None,
                };
                self.start_codeblock(lang, indent)
            }
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_item(),
            Tag::Table(_) => self.start_table(),
            Tag::TableHead => self.start_table_head(),
            Tag::TableRow => self.start_table_row(),
            Tag::TableCell => self.start_table_cell(),
            Tag::Emphasis => self.push_inline_style(self.styles.emphasis),
            Tag::Strong => self.push_inline_style(self.styles.strong),
            Tag::Strikethrough => self.push_inline_style(self.styles.strikethrough),
            Tag::Link { dest_url, .. } => self.push_link(dest_url.to_string()),
            Tag::HtmlBlock
            | Tag::FootnoteDefinition(_)
            | Tag::Image { .. }
            | Tag::MetadataBlock(_) => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Heading(_) => self.end_heading(),
            TagEnd::BlockQuote => self.end_blockquote(),
            TagEnd::CodeBlock => self.end_codeblock(),
            TagEnd::List(_) => self.end_list(),
            TagEnd::Item => {
                self.indent_stack.pop();
                self.pending_marker_line = false;
                self.pending_list_lead_highlight = false;
            }
            TagEnd::Table => self.end_table(),
            TagEnd::TableHead => self.end_table_head(),
            TagEnd::TableRow => self.end_table_row(),
            TagEnd::TableCell => self.end_table_cell(),
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => self.pop_inline_style(),
            TagEnd::Link => self.pop_link(),
            TagEnd::HtmlBlock
            | TagEnd::FootnoteDefinition
            | TagEnd::Image
            | TagEnd::MetadataBlock(_) => {}
        }
    }

    fn start_paragraph(&mut self) {
        if self.needs_newline {
            self.push_blank_line();
        }
        self.push_line(Line::default());
        self.needs_newline = false;
        self.in_paragraph = true;
    }

    fn end_paragraph(&mut self) {
        self.needs_newline = true;
        self.in_paragraph = false;
        self.pending_marker_line = false;
        self.pending_list_lead_highlight = false;
    }

    fn start_heading(&mut self, level: HeadingLevel) {
        if self.needs_newline {
            self.push_line(Line::default());
            self.needs_newline = false;
        }
        let heading_style = match level {
            HeadingLevel::H1 => self.styles.h1,
            HeadingLevel::H2 => self.styles.h2,
            HeadingLevel::H3 => self.styles.h3,
            HeadingLevel::H4 => self.styles.h4,
            HeadingLevel::H5 => self.styles.h5,
            HeadingLevel::H6 => self.styles.h6,
        };
        let content = format!("{} ", "#".repeat(level as usize));
        self.push_line(Line::from(vec![Span::styled(content, heading_style)]));
        self.push_inline_style(heading_style);
        self.needs_newline = false;
    }

    fn end_heading(&mut self) {
        self.needs_newline = true;
        self.pop_inline_style();
    }

    fn start_blockquote(&mut self) {
        if self.needs_newline {
            self.push_blank_line();
            self.needs_newline = false;
        }
        self.indent_stack.push(IndentContext::new(
            vec![Span::from("> ")],
            /*marker*/ None,
            /*is_list*/ false,
        ));
    }

    fn end_blockquote(&mut self) {
        self.indent_stack.pop();
        self.needs_newline = true;
    }

    fn text(&mut self, text: CowStr<'a>) {
        if self.table_state.is_some() {
            self.push_table_text(&text.replace('\n', " "));
            return;
        }
        if self.suppressing_local_link_label() {
            return;
        }
        self.line_ends_with_local_link_target = false;
        if self.pending_marker_line {
            self.push_line(Line::default());
        }
        self.pending_marker_line = false;

        // When inside a fenced code block with a known language, accumulate
        // text into the buffer for batch highlighting in end_codeblock().
        // Append verbatim — pulldown-cmark text events already contain the
        // original line breaks, so inserting separators would double them.
        if self.in_code_block && self.code_block_lang.is_some() {
            self.code_block_buffer.push_str(&text);
            return;
        }

        if self.in_code_block && !self.needs_newline {
            let has_content = self
                .current_line_content
                .as_ref()
                .map(|line| !line.spans.is_empty())
                .unwrap_or_else(|| {
                    self.text
                        .lines
                        .last()
                        .map(|line| !line.spans.is_empty())
                        .unwrap_or(false)
                });
            if has_content {
                self.push_line(Line::default());
            }
        }
        for (i, line) in text.lines().enumerate() {
            if self.needs_newline {
                self.push_line(Line::default());
                self.needs_newline = false;
            }
            if i > 0 {
                self.push_line(Line::default());
            }
            if self.try_render_list_lead(line) {
                continue;
            }
            self.push_text_with_math_highlighting(line);
        }
        self.needs_newline = false;
    }

    fn code(&mut self, code: CowStr<'a>) {
        if self.table_state.is_some() {
            self.push_table_text(&code);
            return;
        }
        if self.suppressing_local_link_label() {
            return;
        }
        self.line_ends_with_local_link_target = false;
        if self.pending_marker_line {
            self.push_line(Line::default());
            self.pending_marker_line = false;
        }
        let span = Span::from(code.into_string()).style(self.styles.code);
        self.push_span(span);
    }

    fn html(&mut self, html: CowStr<'a>, inline: bool) {
        if self.table_state.is_some() {
            self.push_table_text(&html.replace('\n', " "));
            return;
        }
        if self.suppressing_local_link_label() {
            return;
        }
        self.line_ends_with_local_link_target = false;
        self.pending_marker_line = false;
        for (i, line) in html.lines().enumerate() {
            if self.needs_newline {
                self.push_line(Line::default());
                self.needs_newline = false;
            }
            if i > 0 {
                self.push_line(Line::default());
            }
            let style = self
                .inline_styles
                .last()
                .copied()
                .unwrap_or_else(|| Style::default().fg(opencode_markdown_text()));
            self.push_span(Span::styled(line.to_string(), style));
        }
        self.needs_newline = !inline;
    }

    fn hard_break(&mut self) {
        if self.table_state.is_some() {
            self.push_table_text(" ");
            return;
        }
        if self.suppressing_local_link_label() {
            return;
        }
        self.line_ends_with_local_link_target = false;
        self.push_line(Line::default());
    }

    fn soft_break(&mut self) {
        if self.table_state.is_some() {
            self.push_table_text(" ");
            return;
        }
        if self.suppressing_local_link_label() {
            return;
        }
        if self.line_ends_with_local_link_target {
            self.pending_local_link_soft_break = true;
            self.line_ends_with_local_link_target = false;
            return;
        }
        self.line_ends_with_local_link_target = false;
        self.push_line(Line::default());
    }

    fn start_list(&mut self, index: Option<u64>) {
        if self.list_indices.is_empty() && self.needs_newline {
            self.push_line(Line::default());
        }
        self.list_indices.push(index);
    }

    fn end_list(&mut self) {
        self.list_indices.pop();
        self.needs_newline = true;
    }

    fn start_item(&mut self) {
        self.pending_marker_line = true;
        self.pending_list_lead_highlight = true;
        let depth = self.list_indices.len();
        let is_ordered = self
            .list_indices
            .last()
            .map(Option::is_some)
            .unwrap_or(false);
        let width = depth * 4 - 3;
        let marker = if let Some(last_index) = self.list_indices.last_mut() {
            match last_index {
                None => Some(vec![Span::styled(
                    " ".repeat(width - 1) + "- ",
                    self.styles.unordered_list_marker,
                )]),
                Some(index) => {
                    *index += 1;
                    Some(vec![Span::styled(
                        format!("{:width$}. ", *index - 1),
                        self.styles.ordered_list_marker,
                    )])
                }
            }
        } else {
            None
        };
        let indent_prefix = if depth == 0 {
            Vec::new()
        } else {
            let indent_len = if is_ordered { width + 2 } else { width + 1 };
            vec![Span::from(" ".repeat(indent_len))]
        };
        self.indent_stack.push(IndentContext::new(
            indent_prefix,
            marker,
            /*is_list*/ true,
        ));
        self.needs_newline = false;
    }

    fn task_list_marker(&mut self, checked: bool) {
        if self.table_state.is_some() {
            self.push_table_text(if checked { "[x] " } else { "[ ] " });
            return;
        }
        if self.pending_marker_line {
            self.push_line(Line::default());
            self.pending_marker_line = false;
        }
        self.push_span(Span::styled(
            if checked { "[x] " } else { "[ ] " },
            self.styles.strong,
        ));
    }

    fn start_table(&mut self) {
        self.flush_current_line();
        if !self.text.lines.is_empty() {
            self.push_blank_line();
        }
        self.table_state = Some(TableState::default());
        self.needs_newline = false;
    }

    fn end_table(&mut self) {
        if let Some(table) = self.table_state.take() {
            self.flush_current_line();
            for line in self.render_table(&table) {
                self.text.lines.push(line);
            }
            self.needs_newline = true;
        }
    }

    fn start_table_head(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            table.current_row.clear();
            table.current_cell.clear();
            table.in_header = true;
            table.current_row_is_header = true;
        }
    }

    fn end_table_head(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            Self::finish_table_row(table);
            table.in_header = false;
            table.current_row_is_header = false;
        }
    }

    fn start_table_row(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            table.current_row.clear();
            table.current_row_is_header = table.in_header;
        }
    }

    fn end_table_row(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            Self::finish_table_row(table);
        }
    }

    fn start_table_cell(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            table.current_cell.clear();
        }
    }

    fn end_table_cell(&mut self) {
        if let Some(table) = self.table_state.as_mut() {
            table.current_row.push(
                table
                    .current_cell
                    .trim()
                    .replace(TABLE_PIPE_SENTINEL, "|"),
            );
            table.current_cell.clear();
        }
    }

    fn finish_table_row(table: &mut TableState) {
        if table.current_row.is_empty() {
            return;
        }

        let row = std::mem::take(&mut table.current_row);
        table.rows.push((table.current_row_is_header, row));
    }

    fn start_codeblock(&mut self, lang: Option<String>, indent: Option<Span<'static>>) {
        self.flush_current_line();
        if !self.text.lines.is_empty() {
            self.push_blank_line();
        }
        self.in_code_block = true;

        // Extract the language token from the info string.  CommonMark info
        // strings can contain metadata after the language, separated by commas,
        // spaces, or other delimiters (e.g. "rust,no_run", "rust title=demo").
        // Take only the first token so the syntax lookup succeeds.
        let lang = lang
            .as_deref()
            .and_then(|s| s.split([',', ' ', '\t']).next())
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string);
        self.code_block_lang = lang;
        self.code_block_buffer.clear();

        self.indent_stack.push(IndentContext::new(
            vec![indent.unwrap_or_default()],
            /*marker*/ None,
            /*is_list*/ false,
        ));
        self.needs_newline = true;
    }

    fn end_codeblock(&mut self) {
        // If we buffered code for a known language, syntax-highlight it now.
        if let Some(lang) = self.code_block_lang.take() {
            let code = std::mem::take(&mut self.code_block_buffer);
            if !code.is_empty() {
                let highlighted = highlight_code_to_lines(&code, &lang);
                for hl_line in highlighted {
                    self.push_line(Line::default());
                    for span in hl_line.spans {
                        self.push_span(span);
                    }
                }
            }
        }

        self.needs_newline = true;
        self.in_code_block = false;
        self.indent_stack.pop();
    }

    fn push_inline_style(&mut self, style: Style) {
        let current = self.inline_styles.last().copied().unwrap_or_default();
        let merged = current.patch(style);
        self.inline_styles.push(merged);
    }

    fn pop_inline_style(&mut self) {
        self.inline_styles.pop();
    }

    fn push_link(&mut self, dest_url: String) {
        let show_destination = should_render_link_destination(&dest_url);
        self.link = Some(LinkState {
            show_destination,
            local_target_display: if is_local_path_like_link(&dest_url) {
                render_local_link_target(&dest_url, self.cwd.as_deref())
            } else {
                None
            },
            destination: dest_url,
        });
        self.push_inline_style(self.styles.link_text);
    }

    fn pop_link(&mut self) {
        self.pop_inline_style();
        if let Some(link) = self.link.take() {
            if self.table_state.is_some() {
                if link.show_destination {
                    self.push_table_text(" (");
                    self.push_table_text(&link.destination);
                    self.push_table_text(")");
                } else if let Some(local_target_display) = link.local_target_display {
                    self.push_table_text(&local_target_display);
                }
                return;
            }
            if link.show_destination {
                self.push_span(" (".into());
                self.push_span(Span::styled(link.destination, self.styles.link_destination));
                self.push_span(")".into());
            } else if let Some(local_target_display) = link.local_target_display {
                if self.pending_marker_line {
                    self.push_line(Line::default());
                }
                // Local file links are rendered as code-like path text so the transcript shows the
                // resolved target instead of arbitrary caller-provided label text.
                let style = self
                    .inline_styles
                    .last()
                    .copied()
                    .unwrap_or_default()
                    .patch(self.styles.code);
                self.push_span(Span::styled(local_target_display, style));
                self.line_ends_with_local_link_target = true;
            }
        }
    }

    fn suppressing_local_link_label(&self) -> bool {
        self.link
            .as_ref()
            .and_then(|link| link.local_target_display.as_ref())
            .is_some()
    }

    fn push_table_text(&mut self, text: &str) {
        if let Some(table) = self.table_state.as_mut() {
            let sanitized = if text.contains(TABLE_PIPE_SENTINEL) {
                text.replace(TABLE_PIPE_SENTINEL, "|")
            } else {
                text.to_string()
            };
            table.current_cell.push_str(&sanitized);
        }
    }

    fn push_text_with_math_highlighting(&mut self, line: &str) {
        let base_style = self.base_inline_style();
        let math_style = Style::default().fg(opencode_accent());
        let trimmed = line.trim();
        if trimmed == "$$" {
            self.push_span(Span::styled(line.to_string(), math_style));
            self.in_math_block = !self.in_math_block;
            return;
        }
        if self.in_math_block {
            self.push_span(Span::styled(line.to_string(), math_style));
            return;
        }

        let mut cursor = 0usize;
        while let Some(start_rel) = line[cursor..].find('$') {
            let start = cursor + start_rel;
            let delimiter_len = if line[start..].starts_with("$$") { 2 } else { 1 };
            let search_from = start + delimiter_len;
            let closing = line[search_from..]
                .find(if delimiter_len == 2 { "$$" } else { "$" })
                .map(|idx| search_from + idx);

            let Some(end) = closing else {
                break;
            };

            if start > cursor {
                self.push_span(Span::styled(line[cursor..start].to_string(), base_style));
            }

            let end_exclusive = end + delimiter_len;
            self.push_span(Span::styled(
                line[start..end_exclusive].to_string(),
                math_style,
            ));
            cursor = end_exclusive;
        }

        if cursor < line.len() {
            self.push_span(Span::styled(line[cursor..].to_string(), base_style));
        }
    }

    fn base_inline_style(&self) -> Style {
        self.inline_styles
            .last()
            .copied()
            .unwrap_or_else(|| Style::default().fg(opencode_markdown_text()))
    }

    fn try_render_list_lead(&mut self, line: &str) -> bool {
        if !self.pending_list_lead_highlight || line.trim().is_empty() || !self.inline_styles.is_empty() {
            return false;
        }

        let base_style = self.base_inline_style();
        let highlight_style = self.styles.strong;
        let trimmed = line.trim_start();
        let leading_ws = line.len().saturating_sub(trimmed.len());

        if leading_ws > 0 {
            self.push_span(Span::styled(line[..leading_ws].to_string(), base_style));
        }

        if let Some(colon_idx) = trimmed.find(['：', ':']).filter(|idx| *idx > 0 && *idx <= 20) {
            let label = &trimmed[..colon_idx];
            let label_char_count = label.chars().count();
            let looks_like_label = label_char_count <= 12
                && !label.chars().any(|c| {
                    matches!(c, '，' | ',' | '。' | '.' | '？' | '?' | '！' | '!' | '；' | ';')
                })
                && !label.chars().any(char::is_whitespace)
                && !label.starts_with('[');
            if !looks_like_label {
                self.pending_list_lead_highlight = false;
                return false;
            }
            let delimiter_width = trimmed[colon_idx..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
            let split = colon_idx + delimiter_width;
            self.push_span(Span::styled(trimmed[..split].to_string(), highlight_style));
            if split < trimmed.len() {
                self.push_span(Span::styled(trimmed[split..].to_string(), base_style));
            }
            self.pending_list_lead_highlight = false;
            return true;
        }

        self.pending_list_lead_highlight = false;
        false
    }

    fn flush_current_line(&mut self) {
        if let Some(line) = self.current_line_content.take() {
            let style = self.current_line_style;
            // NB we don't wrap code in code blocks, in order to preserve whitespace for copy/paste.
            if !self.current_line_in_code_block
                && let Some(width) = self.wrap_width
            {
                let opts = RtOptions::new(width)
                    .initial_indent(self.current_initial_indent.clone().into())
                    .subsequent_indent(self.current_subsequent_indent.clone().into());
                for wrapped in adaptive_wrap_line(&line, opts) {
                    let owned = line_to_static(&wrapped).style(style);
                    self.text.lines.push(owned);
                }
            } else {
                let mut spans = self.current_initial_indent.clone();
                let mut line = line;
                spans.append(&mut line.spans);
                self.text.lines.push(Line::from_iter(spans).style(style));
            }
            self.current_initial_indent.clear();
            self.current_subsequent_indent.clear();
            self.current_line_in_code_block = false;
            self.line_ends_with_local_link_target = false;
        }
    }

    fn render_table(&self, table: &TableState) -> Vec<Line<'static>> {
        if table.rows.is_empty() {
            return Vec::new();
        }

        let col_count = table.rows.iter().map(|(_, row)| row.len()).max().unwrap_or(0);
        if col_count == 0 {
            return Vec::new();
        }

        let mut widths = vec![0usize; col_count];
        for (_, row) in &table.rows {
            for (idx, cell) in row.iter().enumerate() {
                widths[idx] = widths[idx].max(UnicodeWidthStr::width(cell.as_str()).max(2));
            }
        }
        let widths = self.fit_table_widths(&widths);

        let border_style = Style::default().fg(opencode_text_muted());
        let header_style = Style::default()
            .fg(opencode_markdown_link())
            .add_modifier(ratatui::style::Modifier::BOLD);
        let body_style = Style::default().fg(opencode_markdown_text());

        let mut out = Vec::new();
        out.push(self.render_table_border('┌', '┬', '┐', &widths, border_style));

        for (row_idx, (is_header, row)) in table.rows.iter().enumerate() {
            out.extend(self.render_table_row(
                row,
                &widths,
                if *is_header { header_style } else { body_style },
                border_style,
            ));
            if row_idx + 1 < table.rows.len() {
                out.push(self.render_table_border('├', '┼', '┤', &widths, border_style));
            }
        }

        out.push(self.render_table_border('└', '┴', '┘', &widths, border_style));
        out
    }

    fn render_table_border(
        &self,
        left: char,
        mid: char,
        right: char,
        widths: &[usize],
        style: Style,
    ) -> Line<'static> {
        let mut text = String::new();
        text.push(left);
        for (idx, width) in widths.iter().enumerate() {
            text.push_str(&"─".repeat(width + 2));
            text.push(if idx + 1 == widths.len() { right } else { mid });
        }
        Line::from(vec![Span::styled(text, style)])
    }

    fn fit_table_widths(&self, natural_widths: &[usize]) -> Vec<usize> {
        let Some(max_width) = self.wrap_width else {
            return natural_widths.to_vec();
        };
        if natural_widths.is_empty() {
            return Vec::new();
        }

        let border_width = natural_widths.len() * 3 + 1;
        let content_budget = max_width.saturating_sub(border_width);
        if content_budget == 0 {
            return vec![1; natural_widths.len()];
        }

        let mut widths = natural_widths.to_vec();
        let mut min_widths: Vec<usize> = natural_widths
            .iter()
            .map(|width| (*width).min(4).max(1))
            .collect();

        if min_widths.iter().sum::<usize>() > content_budget {
            min_widths.fill(1);
        }

        while widths.iter().sum::<usize>() > content_budget {
            let Some((idx, _)) = widths
                .iter()
                .enumerate()
                .filter(|(idx, width)| **width > min_widths[*idx])
                .max_by_key(|(_, width)| **width)
            else {
                break;
            };
            widths[idx] -= 1;
        }

        if widths.iter().sum::<usize>() <= content_budget {
            return widths;
        }

        let mut widths = vec![1; natural_widths.len()];
        let mut remaining = content_budget.saturating_sub(widths.len());
        let mut indices: Vec<usize> = (0..natural_widths.len()).collect();
        indices.sort_by_key(|idx| std::cmp::Reverse(natural_widths[*idx]));

        while remaining > 0 {
            let mut progressed = false;
            for idx in &indices {
                if widths[*idx] < natural_widths[*idx] {
                    widths[*idx] += 1;
                    remaining -= 1;
                    progressed = true;
                    if remaining == 0 {
                        break;
                    }
                }
            }
            if !progressed {
                break;
            }
        }

        widths
    }

    fn wrap_table_cell(&self, cell: &str, width: usize) -> Vec<String> {
        if cell.is_empty() {
            return vec![String::new()];
        }
        let width = width.max(1);
        let line = Line::from(cell.to_string());
        let wrapped = adaptive_wrap_line(&line, RtOptions::new(width));
        let mut result: Vec<String> = wrapped
            .into_iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect();
        if result.is_empty() {
            result.push(String::new());
        }
        result
    }

    fn render_table_row(
        &self,
        row: &[String],
        widths: &[usize],
        cell_style: Style,
        border_style: Style,
    ) -> Vec<Line<'static>> {
        let wrapped_cells: Vec<Vec<String>> = widths
            .iter()
            .enumerate()
            .map(|(idx, width)| {
                let cell = row.get(idx).map(String::as_str).unwrap_or_default();
                self.wrap_table_cell(cell, *width)
            })
            .collect();
        let row_height = wrapped_cells.iter().map(Vec::len).max().unwrap_or(1);

        let mut lines = Vec::with_capacity(row_height);
        for line_idx in 0..row_height {
            let mut spans = Vec::with_capacity(widths.len() * 4 + 1);
            spans.push(Span::styled("│", border_style));
            for (idx, width) in widths.iter().enumerate() {
                let cell_line = wrapped_cells[idx]
                    .get(line_idx)
                    .map(String::as_str)
                    .unwrap_or_default();
                let padding = width.saturating_sub(UnicodeWidthStr::width(cell_line));
                spans.push(Span::styled(" ", border_style));
                spans.push(Span::styled(
                    format!("{cell_line}{}", " ".repeat(padding)),
                    cell_style,
                ));
                spans.push(Span::styled(" ", border_style));
                spans.push(Span::styled("│", border_style));
            }
            lines.push(Line::from(spans));
        }

        lines
    }

    fn push_line(&mut self, line: Line<'static>) {
        self.flush_current_line();
        let blockquote_active = self
            .indent_stack
            .iter()
            .any(|ctx| ctx.prefix.iter().any(|s| s.content.contains('>')));
        let style = if self.in_code_block {
            line.style
                .patch(Style::default().bg(opencode_code_block_background()))
        } else if blockquote_active {
            self.styles.blockquote
        } else {
            line.style
        };
        let was_pending = self.pending_marker_line;

        self.current_initial_indent = self.prefix_spans(was_pending);
        self.current_subsequent_indent = self.prefix_spans(/*pending_marker_line*/ false);
        self.current_line_style = style;
        self.current_line_content = Some(line);
        self.current_line_in_code_block = self.in_code_block;
        self.line_ends_with_local_link_target = false;

        self.pending_marker_line = false;
    }

    fn push_span(&mut self, span: Span<'static>) {
        if let Some(line) = self.current_line_content.as_mut() {
            line.push_span(span);
        } else {
            self.push_line(Line::from(vec![span]));
        }
    }

    fn push_blank_line(&mut self) {
        self.flush_current_line();
        if self.indent_stack.iter().all(|ctx| ctx.is_list) {
            self.text.lines.push(Line::default());
        } else {
            self.push_line(Line::default());
            self.flush_current_line();
        }
    }

    fn prefix_spans(&self, pending_marker_line: bool) -> Vec<Span<'static>> {
        let mut prefix: Vec<Span<'static>> = Vec::new();
        let last_marker_index = if pending_marker_line {
            self.indent_stack
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, ctx)| if ctx.marker.is_some() { Some(i) } else { None })
        } else {
            None
        };
        let last_list_index = self.indent_stack.iter().rposition(|ctx| ctx.is_list);

        for (i, ctx) in self.indent_stack.iter().enumerate() {
            if pending_marker_line {
                if Some(i) == last_marker_index
                    && let Some(marker) = &ctx.marker
                {
                    prefix.extend(marker.iter().cloned());
                    continue;
                }
                if ctx.is_list && last_marker_index.is_some_and(|idx| idx > i) {
                    continue;
                }
            } else if ctx.is_list && Some(i) != last_list_index {
                continue;
            }
            prefix.extend(ctx.prefix.iter().cloned());
        }

        prefix
    }
}

fn is_local_path_like_link(dest_url: &str) -> bool {
    dest_url.starts_with("file://")
        || dest_url.starts_with('/')
        || dest_url.starts_with("~/")
        || dest_url.starts_with("./")
        || dest_url.starts_with("../")
        || dest_url.starts_with("\\\\")
        || matches!(
            dest_url.as_bytes(),
            [drive, b':', separator, ..]
                if drive.is_ascii_alphabetic() && matches!(separator, b'/' | b'\\')
        )
}

/// Parse a local link target into normalized path text plus an optional location suffix.
///
/// This accepts the path shapes Codex emits today: `file://` URLs, absolute and relative paths,
/// `~/...`, Windows paths, and `#L..C..` or `:line:col` suffixes.
fn render_local_link_target(dest_url: &str, cwd: Option<&Path>) -> Option<String> {
    let (path_text, location_suffix) = parse_local_link_target(dest_url)?;
    let mut rendered = display_local_link_path(&path_text, cwd);
    if let Some(location_suffix) = location_suffix {
        rendered.push_str(&location_suffix);
    }
    Some(rendered)
}

/// Split a local-link destination into `(normalized_path_text, location_suffix)`.
///
/// The returned path text never includes a trailing `#L..` or `:line[:col]` suffix. Path
/// normalization expands `~/...` when possible and rewrites path separators into display-stable
/// forward slashes. The suffix, when present, is returned separately in normalized markdown form.
///
/// Returns `None` only when the destination looks like a `file://` URL but cannot be parsed into a
/// local path. Plain path-like inputs always return `Some(...)` even if they are relative.
fn parse_local_link_target(dest_url: &str) -> Option<(String, Option<String>)> {
    if dest_url.starts_with("file://") {
        let url = Url::parse(dest_url).ok()?;
        let path_text = file_url_to_local_path_text(&url)?;
        let location_suffix = url
            .fragment()
            .and_then(normalize_hash_location_suffix_fragment);
        return Some((path_text, location_suffix));
    }

    let mut path_text = dest_url;
    let mut location_suffix = None;
    // Prefer `#L..` style fragments when both forms are present so URLs like `path#L10` do not
    // get misparsed as a plain path ending in `:10`.
    if let Some((candidate_path, fragment)) = dest_url.rsplit_once('#')
        && let Some(normalized) = normalize_hash_location_suffix_fragment(fragment)
    {
        path_text = candidate_path;
        location_suffix = Some(normalized);
    }
    if location_suffix.is_none()
        && let Some(suffix) = extract_colon_location_suffix(path_text)
    {
        let path_len = path_text.len().saturating_sub(suffix.len());
        path_text = &path_text[..path_len];
        location_suffix = Some(suffix);
    }

    Some((expand_local_link_path(path_text), location_suffix))
}

/// Normalize a hash fragment like `L12` or `L12C3-L14C9` into the display suffix we render.
///
/// Returns `None` for fragments that are not location references. This deliberately ignores other
/// `#...` fragments so non-location hashes stay part of the path text.
fn normalize_hash_location_suffix_fragment(fragment: &str) -> Option<String> {
    HASH_LOCATION_SUFFIX_RE
        .is_match(fragment)
        .then(|| format!("#{fragment}"))
        .and_then(|suffix| normalize_markdown_hash_location_suffix(&suffix))
}

/// Extract a trailing `:line`, `:line:col`, or range suffix from a plain path-like string.
///
/// The suffix must occur at the end of the input; embedded colons elsewhere in the path are left
/// alone. This is what keeps Windows drive letters like `C:/...` from being misread as locations.
fn extract_colon_location_suffix(path_text: &str) -> Option<String> {
    COLON_LOCATION_SUFFIX_RE
        .find(path_text)
        .filter(|matched| matched.end() == path_text.len())
        .map(|matched| matched.as_str().to_string())
}

/// Expand home-relative paths and normalize separators for display.
///
/// If `~/...` cannot be expanded because the home directory is unavailable, the original text still
/// goes through separator normalization and is returned as-is otherwise.
fn expand_local_link_path(path_text: &str) -> String {
    // Expand `~/...` eagerly so home-relative links can participate in the same normalization and
    // cwd-relative shortening path as absolute links.
    if let Some(rest) = path_text.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return normalize_local_link_path_text(&home.join(rest).to_string_lossy());
    }

    normalize_local_link_path_text(path_text)
}

/// Convert a `file://` URL into the normalized local-path text used for transcript rendering.
///
/// This prefers `Url::to_file_path()` for standard file URLs. When that rejects Windows-oriented
/// encodings, we reconstruct a display path from the host/path parts so UNC paths and drive-letter
/// URLs still render sensibly.
fn file_url_to_local_path_text(url: &Url) -> Option<String> {
    if let Ok(path) = url.to_file_path() {
        return Some(normalize_local_link_path_text(&path.to_string_lossy()));
    }

    // Fall back to string reconstruction for cases `to_file_path()` rejects, especially UNC-style
    // hosts and Windows drive paths encoded in URL form.
    let mut path_text = url.path().to_string();
    if let Some(host) = url.host_str()
        && !host.is_empty()
        && host != "localhost"
    {
        path_text = format!("//{host}{path_text}");
    } else if matches!(
        path_text.as_bytes(),
        [b'/', drive, b':', b'/', ..] if drive.is_ascii_alphabetic()
    ) {
        path_text.remove(0);
    }

    Some(normalize_local_link_path_text(&path_text))
}

/// Normalize local-path text into the transcript display form.
///
/// Display normalization is intentionally lexical: it does not touch the filesystem, resolve
/// symlinks, or collapse `.` / `..`. It only converts separators to forward slashes and rewrites
/// UNC-style `\\\\server\\share` inputs into `//server/share` so later prefix checks operate on a
/// stable representation.
fn normalize_local_link_path_text(path_text: &str) -> String {
    // Render all local link paths with forward slashes so display and prefix stripping are stable
    // across mixed Windows and Unix-style inputs.
    if let Some(rest) = path_text.strip_prefix("\\\\") {
        format!("//{}", rest.replace('\\', "/").trim_start_matches('/'))
    } else {
        path_text.replace('\\', "/")
    }
}

fn is_absolute_local_link_path(path_text: &str) -> bool {
    path_text.starts_with('/')
        || path_text.starts_with("//")
        || matches!(
            path_text.as_bytes(),
            [drive, b':', b'/', ..] if drive.is_ascii_alphabetic()
        )
}

/// Remove trailing separators from a local path without destroying root semantics.
///
/// Roots like `/`, `//`, and `C:/` stay intact so callers can still distinguish "the root itself"
/// from "a path under the root".
fn trim_trailing_local_path_separator(path_text: &str) -> &str {
    if path_text == "/" || path_text == "//" {
        return path_text;
    }
    if matches!(path_text.as_bytes(), [drive, b':', b'/'] if drive.is_ascii_alphabetic()) {
        return path_text;
    }
    path_text.trim_end_matches('/')
}

/// Strip `cwd_text` from the start of `path_text` when `path_text` is strictly underneath it.
///
/// Returns the relative remainder without a leading slash. If the path equals the cwd exactly, this
/// returns `None` so callers can keep rendering the full path instead of collapsing it to an empty
/// string.
fn strip_local_path_prefix<'a>(path_text: &'a str, cwd_text: &str) -> Option<&'a str> {
    let path_text = trim_trailing_local_path_separator(path_text);
    let cwd_text = trim_trailing_local_path_separator(cwd_text);
    if path_text == cwd_text {
        return None;
    }

    // Treat filesystem roots specially so `/tmp/x` under `/` becomes `tmp/x` instead of being
    // left unchanged by the generic prefix-stripping branch.
    if cwd_text == "/" || cwd_text == "//" {
        return path_text.strip_prefix('/');
    }

    path_text
        .strip_prefix(cwd_text)
        .and_then(|rest| rest.strip_prefix('/'))
}

/// Choose the visible path text for a local link after normalization.
///
/// Relative paths stay relative. Absolute paths are shortened against `cwd` only when they are
/// lexically underneath it; otherwise the absolute path is preserved. This is display logic only,
/// not filesystem canonicalization.
fn display_local_link_path(path_text: &str, cwd: Option<&Path>) -> String {
    let path_text = normalize_local_link_path_text(path_text);
    if !is_absolute_local_link_path(&path_text) {
        return path_text;
    }

    if let Some(cwd) = cwd {
        // Only shorten absolute paths that are under the provided session cwd; otherwise preserve
        // the original absolute target for clarity.
        let cwd_text = normalize_local_link_path_text(&cwd.to_string_lossy());
        if let Some(stripped) = strip_local_path_prefix(&path_text, &cwd_text) {
            return stripped.to_string();
        }
    }

    path_text
}

#[cfg(test)]
mod markdown_render_tests {
    include!("markdown_render_tests.rs");
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::text::Text;

    fn lines_to_strings(text: &Text<'_>) -> Vec<String> {
        text.lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.clone())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn wraps_plain_text_when_width_provided() {
        let markdown = "This is a simple sentence that should wrap.";
        let rendered = render_markdown_text_with_width(markdown, Some(16));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "This is a simple".to_string(),
                "sentence that".to_string(),
                "should wrap.".to_string(),
            ]
        );
    }

    #[test]
    fn wraps_list_items_preserving_indent() {
        let markdown = "- first second third fourth";
        let rendered = render_markdown_text_with_width(markdown, Some(14));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec!["- first second".to_string(), "  third fourth".to_string(),]
        );
    }

    #[test]
    fn wraps_nested_lists() {
        let markdown =
            "- outer item with several words to wrap\n  - inner item that also needs wrapping";
        let rendered = render_markdown_text_with_width(markdown, Some(20));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "- outer item with".to_string(),
                "  several words to".to_string(),
                "  wrap".to_string(),
                "    - inner item".to_string(),
                "      that also".to_string(),
                "      needs wrapping".to_string(),
            ]
        );
    }

    #[test]
    fn wraps_ordered_lists() {
        let markdown = "1. ordered item contains many words for wrapping";
        let rendered = render_markdown_text_with_width(markdown, Some(18));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "1. ordered item".to_string(),
                "   contains many".to_string(),
                "   words for".to_string(),
                "   wrapping".to_string(),
            ]
        );
    }

    #[test]
    fn wraps_blockquotes() {
        let markdown = "> block quote with content that should wrap nicely";
        let rendered = render_markdown_text_with_width(markdown, Some(22));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "> block quote with".to_string(),
                "> content that should".to_string(),
                "> wrap nicely".to_string(),
            ]
        );
    }

    #[test]
    fn wraps_blockquotes_inside_lists() {
        let markdown = "- list item\n  > block quote inside list that wraps";
        let rendered = render_markdown_text_with_width(markdown, Some(24));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "- list item".to_string(),
                "  > block quote inside".to_string(),
                "  > list that wraps".to_string(),
            ]
        );
    }

    #[test]
    fn wraps_list_items_containing_blockquotes() {
        let markdown = "1. item with quote\n   > quoted text that should wrap";
        let rendered = render_markdown_text_with_width(markdown, Some(24));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec![
                "1. item with quote".to_string(),
                "   > quoted text that".to_string(),
                "   > should wrap".to_string(),
            ]
        );
    }

    #[test]
    fn does_not_wrap_code_blocks() {
        let markdown = "````\nfn main() { println!(\"hi from a long line\"); }\n````";
        let rendered = render_markdown_text_with_width(markdown, Some(10));
        let lines = lines_to_strings(&rendered);
        assert_eq!(
            lines,
            vec!["fn main() { println!(\"hi from a long line\"); }".to_string(),]
        );
    }

    #[test]
    fn does_not_split_long_url_like_token_without_scheme() {
        let url_like =
            "example.test/api/v1/projects/alpha-team/releases/2026-02-17/builds/1234567890";
        let rendered = render_markdown_text_with_width(url_like, Some(24));
        let lines = lines_to_strings(&rendered);

        assert_eq!(
            lines.iter().filter(|line| line.contains(url_like)).count(),
            1,
            "expected full URL-like token in one rendered line, got: {lines:?}"
        );
    }

    #[test]
    fn fenced_code_info_string_with_metadata_highlights() {
        // CommonMark info strings like "rust,no_run" or "rust title=demo"
        // contain metadata after the language token.  The language must be
        // extracted (first word / comma-separated token) so highlighting works.
        for info in &["rust,no_run", "rust no_run", "rust title=\"demo\""] {
            let markdown = format!("```{info}\nfn main() {{}}\n```\n");
            let rendered = render_markdown_text(&markdown);
            let has_rgb = rendered.lines.iter().any(|line| {
                line.spans
                    .iter()
                    .any(|s| matches!(s.style.fg, Some(ratatui::style::Color::Rgb(..))))
            });
            assert!(
                has_rgb,
                "info string \"{info}\" should still produce syntax highlighting"
            );
        }
    }

    #[test]
    fn crlf_code_block_no_extra_blank_lines() {
        // pulldown-cmark can split CRLF code blocks into multiple Text events.
        // The buffer must concatenate them verbatim — no inserted separators.
        let markdown = "```rust\r\nfn main() {}\r\n    line2\r\n```\r\n";
        let rendered = render_markdown_text(markdown);
        let lines = lines_to_strings(&rendered);
        // Should be exactly two code lines; no spurious blank line between them.
        assert_eq!(
            lines,
            vec!["fn main() {}".to_string(), "    line2".to_string()],
            "CRLF code block should not produce extra blank lines: {lines:?}"
        );
    }
}
