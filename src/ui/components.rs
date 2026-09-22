use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Padding, Paragraph},
};

use super::theme::*;
use crate::{app::App, keys::Hint};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn draw_header_banner(frame: &mut Frame<'_>, area: Rect, full: bool) {
    if full {
        let logo_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    " ",
                    Style::default().fg(ACCENT_HI).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "A O V R",
                    Style::default().fg(ACCENT_HI).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("      "),
                Span::styled(
                    format!("Toggle Overlays  ·  v{VERSION}"),
                    Style::default().fg(FG_DIM),
                ),
            ]),
        ];
        frame.render_widget(Paragraph::new(logo_lines), area);
    } else {
        let compact_line = Line::from(vec![
            Span::styled("  ", Style::default().fg(ACCENT)),
            Span::styled(
                "AOVR",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  │  Toggle Overlays", Style::default().fg(FG_DIM)),
        ]);
        frame.render_widget(Paragraph::new(compact_line), area);
    }
}

pub fn render_breadcrumb(frame: &mut Frame<'_>, left: &str, right: &str, area: Rect) {
    let gutter = Span::styled(" ▎ ", Style::default().fg(ACCENT));
    let left_line = Line::from(vec![
        gutter,
        Span::styled(
            left.to_string(),
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        ),
    ]);

    let right_line = Line::from(vec![
        Span::styled(" 󰇙 ", Style::default().fg(FG_DIM)),
        Span::styled(
            right.to_string(),
            Style::default().fg(ACCENT_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ]);

    let left_w = left_line.width() as u16;
    let right_w = right_line.width() as u16;
    let need_w = left_w + right_w + 2;

    if area.width >= need_w {
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(right_w)]).areas(area);

        frame.render_widget(Paragraph::new(left_line), left_area);
        frame.render_widget(
            Paragraph::new(right_line).alignment(Alignment::Right),
            right_area,
        );
    } else {
        frame.render_widget(Paragraph::new(left_line), area);
    }
}

pub fn render_footer(frame: &mut Frame<'_>, status: &str, hints: &[Hint], area: Rect) {
    let [sep_area, hint_area, _pad] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);

    let w = sep_area.width as usize;
    let rule = "─".repeat(w);
    frame.render_widget(
        Paragraph::new(Span::styled(rule, Style::default().fg(RULE_FG))),
        sep_area,
    );

    let mut spans: Vec<Span> = vec![Span::raw("  ")];
    let mut used: u16 = 2;

    if !status.is_empty() {
        let max_status_w = ((hint_area.width as usize) * 40 / 100).max(12);
        let msg = truncate_str(status, max_status_w);
        let msg_len = msg.chars().count() as u16;

        spans.push(Span::styled(msg, Style::default().fg(FG_MID)));
        spans.push(Span::styled("   │   ", Style::default().fg(RULE_FG)));

        used += 2 + msg_len + 7;
    }

    for hint in hints {
        let item_w = (hint.key.chars().count() + 1 + hint.desc.chars().count() + 3) as u16;
        if used + item_w > hint_area.width {
            break;
        }
        spans.push(Span::styled(
            hint.key.to_string(),
            Style::default().fg(ACCENT_DIM).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}   ", hint.desc),
            Style::default().fg(FG_DIM),
        ));
        used += item_w;
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), hint_area);
}

pub fn render_search_footer(frame: &mut Frame<'_>, app: &App, hints: &[Hint], area: Rect) {
    if app.searching {
        render_footer(frame, &app.status, crate::keys::SEARCHING, area);
    } else {
        render_footer(frame, &app.status, hints, area);
    }
}

pub fn draw_search_bar(
    frame: &mut Frame<'_>,
    app: &mut App,
    area: Rect,
    scope: &str,
    matches: usize,
) {
    let query = app.active_search_query().to_string();
    let focused = app.searching;
    let border = if focused { ACCENT } else { RULE_FG };

    let title = if query.is_empty() {
        format!(" Search · {scope} ")
    } else {
        format!(
            " Search · {scope} · {matches} match{} ",
            if matches == 1 { "" } else { "es" }
        )
    };
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .padding(Padding::horizontal(1))
        .title(Span::styled(
            title,
            Style::default().fg(if focused { ACCENT_HI } else { FG_DIM }),
        ))
        .bg(PANEL_BG);
    let inner = block.inner(area);
    let [input_area, clear_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(3)]).areas(inner);
    frame.render_widget(block, area);

    let input = if focused {
        Line::from(vec![
            Span::styled(" / ", Style::default().fg(ACCENT)),
            Span::styled(query, Style::default().fg(FG).add_modifier(Modifier::BOLD)),
            Span::styled("▌", Style::default().fg(ACCENT_HI)),
        ])
    } else if query.is_empty() {
        Line::from(vec![Span::styled(
            " /  Filter…  (press / or click)",
            Style::default().fg(FG_DIM),
        )])
    } else {
        Line::from(vec![
            Span::styled(" / ", Style::default().fg(ACCENT)),
            Span::styled(query, Style::default().fg(FG)),
        ])
    };
    frame.render_widget(Paragraph::new(input), input_area);

    if app.active_search_query().is_empty() {
        app.search_clear_area = None;
    } else {
        app.search_clear_area = Some(clear_area);
        frame.render_widget(
            Paragraph::new("×")
                .alignment(Alignment::Center)
                .style(Style::default().fg(FG_DIM)),
            clear_area,
        );
    }
}

pub fn body_block() -> Block<'static> {
    Block::default()
        .bg(PANEL_BG)
        .padding(Padding::new(2, 2, 1, 1))
}

pub fn section_head(icon: &str, label: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{icon}  {label}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

pub fn section_divider() -> Line<'static> {
    Line::from("")
}

pub fn overlay_line(name: &str, selected: bool, enabled: bool) -> Line<'static> {
    let (icon, icon_color) = if enabled {
        ("󰗠", C_OK)
    } else {
        ("󰂭", C_WARN)
    };

    if selected {
        Line::from(vec![
            Span::styled(
                "  ",
                Style::default()
                    .fg(ACCENT)
                    .bg(SEL_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{icon} "),
                Style::default().fg(ACCENT_HI).bg(SEL_BG),
            ),
            Span::styled(
                format!("  {name}"),
                Style::default()
                    .fg(SEL_FG)
                    .bg(SEL_BG)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
            Span::styled(format!("  {name}"), Style::default().fg(FG_MID)),
        ])
    }
}

pub fn key_ref_row<'a>(key: &'a str, desc: &'a str, narrow: bool) -> Line<'a> {
    let col_w = if narrow { 14 } else { 18 };
    let key_padded = format!("{key:<col_w$}");
    Line::from(vec![
        Span::raw("   "),
        Span::styled(
            key_padded,
            Style::default().fg(ACCENT_DIM).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(desc, Style::default().fg(FG_MID)),
    ])
}

pub fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    if max_chars <= 1 {
        return "…".chars().take(max_chars).collect();
    }
    let mut out: String = s.chars().take(max_chars - 1).collect();
    out.push('…');
    out
}
