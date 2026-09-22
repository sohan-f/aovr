use ratatui::{prelude::*, widgets::Paragraph};

use super::super::{components::*, theme::*};
use crate::{
    app::{App, OverlayRow, SEARCH_BAR_H, SEARCH_GAP_H},
    keys,
};

struct SectionRange {
    header: Line<'static>,
    start_line: usize,
    end_line: usize,
}

pub fn draw(frame: &mut Frame<'_>, app: &mut App, crumb: Rect, body: Rect, foot: Rect) {
    let Some(target_name) = app.current_target_name().map(str::to_string) else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  \u{f02fd} ", Style::default().fg(FG_DIM)),
                    Span::styled("No targets loaded.", Style::default().fg(FG_MID)),
                ]),
            ])
            .block(body_block()),
            body,
        );
        return;
    };

    #[cfg(feature = "virtual")]
    let right = if app.is_virtual() {
        "virtual · overlays"
    } else {
        "overlays"
    };
    #[cfg(not(feature = "virtual"))]
    let right = "overlays";
    render_breadcrumb(frame, &format!("\u{f004d}  {target_name}"), right, crumb);

    let actionable = app.visible_overlays();
    let broken = app.visible_broken();

    let [_, search_area, list_area] = Layout::vertical([
        Constraint::Length(SEARCH_GAP_H),
        Constraint::Length(SEARCH_BAR_H),
        Constraint::Fill(1),
    ])
    .areas(body);
    app.search_bar_area = Some(search_area);
    draw_search_bar(frame, app, search_area, "overlays", actionable.len());

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

    append_section("\u{f0132}", "Enabled", C_OK, &enabled_rows, true);
    append_section("\u{f00ad}", "Disabled", C_WARN, &disabled_rows, false);

    if !broken.is_empty() {
        if !lines.is_empty() {
            lines.push(section_divider());
        }
        let start = lines.len();
        let head = section_head("\u{f0026}", "Broken", C_ERR);
        lines.push(head.clone());
        lines.push(section_divider());
        for name in &broken {
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled("\u{f0026} ", Style::default().fg(C_ERR)),
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
        let msg = if app.overlay_search.is_empty() {
            String::from("No overlays for this target.")
        } else {
            format!("No overlays match \"/{}\".", app.overlay_search)
        };
        lines.push(Line::from(vec![
            Span::styled("  \u{f02fd} ", Style::default().fg(FG_DIM)),
            Span::styled(msg, Style::default().fg(FG_MID)),
        ]));
    }

    let block = body_block();
    let inner_area = block.inner(list_area);
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

        frame.render_widget(&block, list_area);
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
            list_area,
        );
    }

    let narrow = list_area.width < NARROW;
    let hints = if narrow {
        keys::DETAIL_NARROW
    } else {
        keys::DETAIL_WIDE
    };
    render_search_footer(frame, app, hints, foot);
}
