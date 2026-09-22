use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Padding, Paragraph},
};

use super::super::{components::*, theme::*};
use crate::{
    app::{App, SEARCH_BAR_H, SEARCH_GAP_H},
    keys,
    screens::Screen,
};

pub fn draw(frame: &mut Frame<'_>, app: &mut App, crumb: Rect, body: Rect, foot: Rect) {
    let indices = app.visible_target_indices();
    let count = indices.len();
    let pos = if count == 0 {
        String::from("empty")
    } else {
        format!("{}/{}", app.selected_target + 1, count)
    };
    let right = if app.is_virtual() {
        format!("virtual · {pos}")
    } else {
        pos
    };
    render_breadcrumb(frame, "󰓾  Targets", &right, crumb);

    let [_, search_area, list_area] = Layout::vertical([
        Constraint::Length(SEARCH_GAP_H),
        Constraint::Length(SEARCH_BAR_H),
        Constraint::Fill(1),
    ])
    .areas(body);
    app.search_bar_area = Some(search_area);
    draw_search_bar(frame, app, search_area, "targets+overlays", count);

    if indices.is_empty() {
        let msg = if app.target_search.is_empty() {
            String::from("No targets found.")
        } else {
            format!("No targets match \"/{}\".", app.target_search)
        };
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  󰋽 ", Style::default().fg(FG_DIM)),
                    Span::styled(msg, Style::default().fg(FG_MID)),
                ]),
            ])
            .block(body_block()),
            list_area,
        );
        render_footer(frame, &app.status, keys::EMPTY, foot);
        return;
    }

    debug_assert!(matches!(app.screen, Screen::Targets));

    let narrow = list_area.width < NARROW;

    let items: Vec<ListItem> = indices
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let name = &app.target_order[idx];
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
                .padding(Padding::new(2, 2, 1, 1)),
        ),
        list_area,
        &mut ls,
    );

    let hints = if narrow {
        keys::TARGETS_NARROW
    } else {
        keys::TARGETS_WIDE
    };
    render_search_footer(frame, app, hints, foot);
}
