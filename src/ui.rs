use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph, Wrap},
};

use crate::{
    app::{App, OverlayRow},
    screens::Screen,
};

const ACCENT: Color = Color::Rgb(157, 122, 240);
const ACCENT_DIM: Color = Color::Rgb(120, 95, 180);
const ACCENT_HI: Color = Color::Rgb(200, 175, 255);
const FG: Color = Color::Rgb(235, 230, 250);
const FG_MID: Color = Color::Rgb(165, 155, 190);
const FG_DIM: Color = Color::Rgb(95, 88, 120);
const PANEL_BG: Color = Color::Rgb(22, 18, 35);
const C_OK: Color = Color::Rgb(80, 220, 150);
const C_WARN: Color = Color::Rgb(240, 180, 60);
const C_ERR: Color = Color::Rgb(240, 90, 90);
const SEL_BG: Color = Color::Rgb(48, 32, 95);
const SEL_FG: Color = Color::Rgb(225, 210, 255);
const RULE_FG: Color = Color::Rgb(55, 45, 75);

const NARROW: u16 = 56;

pub fn draw(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();

    frame.render_widget(Block::default().bg(PANEL_BG), area);

    let banner_h = if area.height >= 22 { 3 } else { 1 };

    let chunks = Layout::vertical([
        Constraint::Length(banner_h), // Header Banner
        Constraint::Length(1),        // Breadcrumb
        Constraint::Fill(1),          // Content body
        Constraint::Length(3),        // Footer
    ])
    .split(area);

    draw_header_banner(frame, chunks[0], banner_h == 3);

    match app.screen {
        Screen::Targets => draw_targets(frame, app, chunks[1], chunks[2], chunks[3]),
        Screen::Detail => draw_detail(frame, app, chunks[1], chunks[2], chunks[3]),
        Screen::About => draw_about(frame, app, chunks[1], chunks[2], chunks[3]),
    }
}

fn draw_header_banner(frame: &mut Frame<'_>, area: Rect, full: bool) {
    if full {
        let logo_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "󰍜 ",
                    Style::default().fg(ACCENT_HI).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "AOVR",
                    Style::default().fg(ACCENT_HI).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default().fg(ACCENT_DIM)),
                Span::styled(
                    "Toggle Overlays",
                    Style::default().fg(FG_DIM).add_modifier(Modifier::BOLD),
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
            Span::styled("  OVERLAY TOGGLE", Style::default().fg(FG_DIM)),
        ]);
        frame.render_widget(Paragraph::new(compact_line), area);
    }
}

fn render_breadcrumb(frame: &mut Frame<'_>, left: &str, right: &str, area: Rect) {
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

fn render_footer(frame: &mut Frame<'_>, status: &str, keys: &[(&str, &str)], area: Rect) {
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

    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    let mut used: u16 = 1;

    if !status.is_empty() {
        let max_status_w = ((hint_area.width as usize) * 40 / 100).max(12);
        let msg = truncate_str(status, max_status_w);
        let msg_len = msg.chars().count() as u16;

        spans.push(Span::styled(msg, Style::default().fg(FG_MID)));
        spans.push(Span::styled("   │   ", Style::default().fg(RULE_FG)));

        used += 2 + msg_len + 7;
    }

    for &(key, desc) in keys {
        let item_w = (key.chars().count() + 1 + desc.chars().count() + 3) as u16;
        if used + item_w > hint_area.width {
            break;
        }
        spans.push(Span::styled(
            key.to_string(),
            Style::default().fg(ACCENT_DIM).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}   "),
            Style::default().fg(FG_DIM),
        ));
        used += item_w;
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), hint_area);
}

fn body_block() -> Block<'static> {
    Block::default()
        .bg(PANEL_BG)
        .padding(Padding::new(2, 2, 1, 1))
}

fn section_head(icon: &str, label: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{icon}  {label}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn section_divider() -> Line<'static> {
    Line::from("")
}

fn overlay_line(name: &str, selected: bool, enabled: bool) -> Line<'static> {
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

fn key_ref_row<'a>(key: &'a str, desc: &'a str, narrow: bool) -> Line<'a> {
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

fn draw_targets(frame: &mut Frame<'_>, app: &App, crumb: Rect, body: Rect, foot: Rect) {
    let count = app.target_order.len();
    let pos = if count == 0 {
        String::from("empty")
    } else {
        format!("{}/{}", app.selected_target + 1, count)
    };
    render_breadcrumb(frame, "󰓾  Targets", &pos, crumb);

    if app.target_order.is_empty() {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  󰋽 ", Style::default().fg(FG_DIM)),
                    Span::styled("No targets found.", Style::default().fg(FG_MID)),
                ]),
            ])
            .block(body_block()),
            body,
        );
        render_footer(frame, &app.status, &[("q", "quit")], foot);
        return;
    }

    let narrow = body.width < NARROW;

    let items: Vec<ListItem> = app
        .target_order
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let total = app.targets.get(name).map(|t| t.total()).unwrap_or(0);
            let sel = i == app.selected_target;
            let bg = if sel { SEL_BG } else { PANEL_BG };

            let prefix = Span::styled(
                if sel { "  " } else { "   " },
                Style::default().fg(ACCENT).bg(bg).add_modifier(if sel {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            );

            let label = Span::styled(
                format!("  {name}"),
                Style::default()
                    .fg(if sel { SEL_FG } else { FG })
                    .bg(bg)
                    .add_modifier(if sel {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );

            let badge_text = if narrow {
                format!("  ×{total}")
            } else {
                format!("   ({total} overlay{})", if total == 1 { "" } else { "s" })
            };
            let badge = Span::styled(
                badge_text,
                Style::default()
                    .fg(if sel { ACCENT_DIM } else { FG_DIM })
                    .bg(bg),
            );

            ListItem::new(Line::from(vec![prefix, label, badge])).bg(bg)
        })
        .collect();

    let mut ls = ListState::default();
    ls.select(Some(app.selected_target));

    frame.render_stateful_widget(
        List::new(items).block(
            Block::default()
                .bg(PANEL_BG)
                .padding(Padding::new(1, 1, 1, 1)),
        ),
        body,
        &mut ls,
    );

    let hints: &[(&str, &str)] = if narrow {
        &[("↑↓", "move"), ("⏎", "open"), ("q", "quit")]
    } else {
        &[("↑↓", "move"), ("⏎", "open"), ("a", "about"), ("q", "quit")]
    };
    render_footer(frame, &app.status, hints, foot);
}

struct SectionRange {
    header: Line<'static>,
    start_line: usize,
    end_line: usize,
}

fn draw_detail(frame: &mut Frame<'_>, app: &App, crumb: Rect, body: Rect, foot: Rect) {
    let Some(target_name) = app.current_target_name() else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  󰋽 ", Style::default().fg(FG_DIM)),
                    Span::styled("No targets loaded.", Style::default().fg(FG_MID)),
                ]),
            ])
            .block(body_block()),
            body,
        );
        return;
    };

    render_breadcrumb(frame, &format!("󰁍  {target_name}"), "overlays", crumb);

    let actionable = app.actionable_overlays();
    let broken = app.broken_overlays();

    let enabled_rows: Vec<(usize, &OverlayRow)> = actionable
        .iter()
        .enumerate()
        .filter(|(_, r)| r.enabled)
        .collect();
    let disabled_rows: Vec<(usize, &OverlayRow)> = actionable
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.enabled)
        .collect();

    let mut lines: Vec<Line> = Vec::new();
    let mut selected_line_idx: Option<usize> = None;
    let mut sections: Vec<SectionRange> = Vec::new();

    let mut append_section = |icon: &'static str,
                              title: &'static str,
                              color: Color,
                              rows: &[(usize, &OverlayRow)],
                              is_enabled: bool| {
        if rows.is_empty() {
            return;
        }
        if !lines.is_empty() {
            lines.push(section_divider());
        }
        let start = lines.len();
        let head = section_head(icon, title, color);
        lines.push(head.clone());
        lines.push(section_divider());
        for (idx, row) in rows {
            if *idx == app.selected_overlay {
                selected_line_idx = Some(lines.len());
            }
            lines.push(overlay_line(
                &row.name,
                *idx == app.selected_overlay,
                is_enabled,
            ));
        }
        sections.push(SectionRange {
            header: head,
            start_line: start,
            end_line: lines.len(),
        });
    };

    append_section("󰄲", "Enabled", C_OK, &enabled_rows, true);
    append_section("󰂭", "Disabled", C_WARN, &disabled_rows, false);

    if !broken.is_empty() {
        if !lines.is_empty() {
            lines.push(section_divider());
        }
        let start = lines.len();
        let head = section_head("󰀦", "Broken", C_ERR);
        lines.push(head.clone());
        lines.push(section_divider());
        for name in &broken {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled("󰀦 ", Style::default().fg(C_ERR)),
                Span::styled(
                    format!("  {name}"),
                    Style::default().fg(C_ERR).add_modifier(Modifier::DIM),
                ),
            ]));
        }
        sections.push(SectionRange {
            header: head,
            start_line: start,
            end_line: lines.len(),
        });
    }

    if actionable.is_empty() && broken.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  󰋽 ", Style::default().fg(FG_DIM)),
            Span::styled("No overlays for this target.", Style::default().fg(FG_MID)),
        ]));
    }

    let block = body_block();
    let inner_area = block.inner(body);
    let view_h = inner_area.height as usize;

    let mut scroll = app.detail_scroll as usize;

    if let Some(target_line) = selected_line_idx
        && view_h > 0
    {
        if target_line < scroll {
            scroll = target_line;
        } else if target_line >= scroll + view_h {
            scroll = target_line.saturating_sub(view_h - 1);
        }
    }

    let sticky_sec = sections
        .iter()
        .find(|s| scroll > s.start_line && scroll < s.end_line);

    if let Some(sec) = sticky_sec {
        let sticky_view_h = view_h.saturating_sub(1);
        if let Some(target_line) = selected_line_idx
            && sticky_view_h > 0
        {
            if target_line < scroll {
                scroll = target_line;
            } else if target_line >= scroll + sticky_view_h {
                scroll = target_line.saturating_sub(sticky_view_h - 1);
            }
        }

        let final_sticky = sections
            .iter()
            .find(|s| scroll > s.start_line && scroll < s.end_line)
            .unwrap_or(sec);

        let [sticky_area, scroll_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner_area);

        frame.render_widget(&block, body);
        frame.render_widget(Paragraph::new(final_sticky.header.clone()), sticky_area);
        frame.render_widget(
            Paragraph::new(lines).scroll((scroll as u16, 0)),
            scroll_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new(lines)
                .block(block)
                .scroll((scroll as u16, 0)),
            body,
        );
    }

    let narrow = body.width < NARROW;
    let hints: &[(&str, &str)] = if narrow {
        &[("↑↓", "move"), ("Spc", "toggle"), ("Esc", "back")]
    } else {
        &[
            ("↑↓", "move"),
            ("Spc", "toggle"),
            ("⏎", "apply+back"),
            ("Esc", "back"),
            ("q", "quit"),
        ]
    };
    render_footer(frame, &app.status, hints, foot);
}

fn draw_about(frame: &mut Frame<'_>, app: &App, crumb: Rect, body: Rect, foot: Rect) {
    render_breadcrumb(frame, "󰍹  About", "AOVR", crumb);

    let narrow = body.width < NARROW;

    let accent = |text| Span::styled(text, Style::default().fg(ACCENT));
    let accent_hi_bold = |text| {
        Span::styled(
            text,
            Style::default().fg(ACCENT_HI).add_modifier(Modifier::BOLD),
        )
    };
    let fg_bold = |text| Span::styled(text, Style::default().fg(FG).add_modifier(Modifier::BOLD));
    let fg_dim = |text| Span::styled(text, Style::default().fg(FG_DIM));
    let fg_mid = |text| Span::styled(text, Style::default().fg(FG_MID));

    let mut content: Vec<Line> = vec![
        Line::from(vec![
            Span::raw("  "),
            accent("󰍜 "),
            accent_hi_bold("AOVR"),
            fg_dim(" : Android Overlay Manager"),
        ]),
        Line::from(vec![
            Span::raw("     "),
            fg_mid("Manage OMS overlays seamlessly via root in Termux."),
        ]),
        section_divider(),
        section_divider(),
        Line::from(vec![
            Span::styled("    ", Style::default().fg(ACCENT)),
            fg_bold("Project & Author"),
        ]),
        section_divider(),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_dim("Developer:  "),
            fg_bold("Sohan Ali"),
        ]),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_dim("Contact:    "),
            fg_mid("sohanakndo019@gmail.com"),
        ]),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_dim("GitHub:     "),
            Span::styled("󰊤 github.com/sohan-f/aovr", Style::default().fg(ACCENT_HI)),
        ]),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_dim("Stack:      "),
            fg_mid("Rust • Ratatui • OMS (su) • MIT"),
        ]),
        section_divider(),
        section_divider(),
        Line::from(vec![
            Span::styled("  󰌌  ", Style::default().fg(ACCENT)),
            fg_bold("Keyboard Reference"),
        ]),
        section_divider(),
    ];

    content.extend([
        key_ref_row("↑ / ↓  j / k", "navigate / scroll page", narrow),
        key_ref_row("PgUp / PgDn", "scroll fast", narrow),
        key_ref_row("Space", "toggle overlay state", narrow),
        key_ref_row("Enter", "open detail / apply changes", narrow),
        key_ref_row("Esc / a", "go back to target list", narrow),
        key_ref_row("q", "quit AOVR", narrow),
    ]);

    content.extend([
        section_divider(),
        section_divider(),
        Line::from(vec![
            Span::styled("  󰌵  ", Style::default().fg(ACCENT)),
            fg_bold("Tips & Notes"),
        ]),
        section_divider(),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_mid("Toggle overlays with Space, press Enter to execute su -c."),
        ]),
        Line::from(vec![
            Span::raw("   "),
            accent("󰅂 "),
            fg_mid("Broken overlays are highlighted and disabled."),
        ]),
    ]);

    let block = body_block();
    let inner_area = block.inner(body);

    let total_lines = content.len() as u16;
    let max_scroll = total_lines.saturating_sub(inner_area.height);
    let scroll_pos = app.about_scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(content).block(block).scroll((scroll_pos, 0)),
        body,
    );

    render_footer(
        frame,
        &app.status,
        &[("j/k", "scroll"), ("a / Esc", "back"), ("q", "quit")],
        foot,
    );
}

/// Safely truncate string `s` to `max_chars` scalar units appending `…` when cut.
fn truncate_str(s: &str, max_chars: usize) -> String {
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
