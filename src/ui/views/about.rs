use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};

use super::super::{components::*, theme::*};
use crate::{app::App, keys};

const HEAD_LINES: u16 = 14;
const TIPS_LINES: u16 = 6;
const VIRTUAL_LINES: u16 = 6;

pub fn content_len(is_virtual: bool) -> u16 {
    HEAD_LINES
        + keys::REFERENCE.len() as u16
        + TIPS_LINES
        + if is_virtual { VIRTUAL_LINES } else { 0 }
}

pub fn draw(frame: &mut Frame<'_>, app: &mut App, crumb: Rect, body: Rect, foot: Rect) {
    app.search_bar_area = None;
    app.search_clear_area = None;
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
            Span::styled("  \u{f007}  ", Style::default().fg(ACCENT)),
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
            Span::styled("  \u{f030c}  ", Style::default().fg(ACCENT)),
            fg_bold("Keyboard Reference"),
        ]),
        section_divider(),
    ];
    debug_assert_eq!(content.len() as u16, HEAD_LINES);

    content.extend(
        keys::REFERENCE
            .iter()
            .map(|hint| key_ref_row(hint.key, hint.desc, narrow)),
    );

    content.extend([
        section_divider(),
        section_divider(),
        Line::from(vec![
            Span::styled("  \u{f0335}  ", Style::default().fg(ACCENT)),
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

    #[cfg(feature = "virtual")]
    if app.is_virtual() {
        content.extend([
            section_divider(),
            section_divider(),
            Line::from(vec![
                Span::styled("   \u{f048d} ", Style::default().fg(ACCENT)),
                fg_bold("Virtual Mode"),
            ]),
            section_divider(),
            Line::from(vec![
                Span::raw("   "),
                accent("󰅂 "),
                fg_mid("Target list replayed from a bundled device capture."),
            ]),
            Line::from(vec![
                Span::raw("   "),
                accent("󰅂 "),
                fg_mid("Toggles update the in-memory list only; no device is touched."),
            ]),
        ]);
    }
    debug_assert_eq!(content.len() as u16, content_len(app.is_virtual()));

    let block: Block = body_block();
    let inner_area = block.inner(body);

    let total_lines = content.len() as u16;
    let max_scroll = total_lines.saturating_sub(inner_area.height);
    let scroll_pos = app.about_scroll.min(max_scroll);

    frame.render_widget(
        Paragraph::new(content).block(block).scroll((scroll_pos, 0)),
        body,
    );

    render_footer(frame, &app.status, keys::ABOUT, foot);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_len_matches_built_content() {
        assert_eq!(content_len(false), 30);
        assert_eq!(content_len(true), 36);
        assert!(!keys::REFERENCE.is_empty());
    }
}
