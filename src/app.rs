use std::{
    io,
    time::{Duration, Instant},
};

use crate::{
    parsers::{TargetOverlays, Targets},
    screens::Screen,
    shell,
};

pub const CHROME_ROWS: u16 = 5;

#[derive(Clone, Debug)]
pub struct OverlayRow {
    pub name: String,
    pub enabled: bool,
}

pub struct App {
    pub screen: Screen,
    pub backend: shell::Backend,
    pub targets: Targets,
    pub target_order: Vec<String>,
    pub selected_target: usize,
    pub selected_overlay: usize,
    pub about_prev: Screen,
    pub status: String,
    /// Shown until replaced instead of expiring after a few seconds.
    pub status_persistent: bool,
    pub status_expires_at: Option<Instant>,
    pub detail_scroll: u16,
    pub about_scroll: u16,
}

impl App {
    pub fn empty() -> Self {
        Self {
            screen: Screen::Targets,
            backend: shell::Backend::default(),
            targets: Targets::default(),
            target_order: Vec::new(),
            selected_target: 0,
            selected_overlay: 0,
            about_prev: Screen::Targets,
            status: String::new(),
            status_persistent: false,
            detail_scroll: 0,
            status_expires_at: None,
            about_scroll: 0,
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_persistent = false;
        self.status_expires_at = None;
    }

    pub fn set_persistent_status(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_persistent = true;
        self.status_expires_at = None;
    }

    pub fn clear_status(&mut self) {
        self.status.clear();
        self.status_persistent = false;
        self.status_expires_at = None;
    }

    pub fn reload(&mut self) {
        match self.backend.list() {
            Ok(targets) => {
                self.targets = targets;
                self.refresh_order();
                let count = self.target_order.len();
                self.set_status(format!(
                    "✓ Loaded {count} target{}",
                    if count == 1 { "" } else { "s" }
                ));
            }
            Err(err) => self.set_persistent_status(format!("Load failed: {err}")),
        }
    }

    /// On failure the chosen backend is kept so `r` retries the same source.
    pub fn load() -> Self {
        let mut backend = shell::Backend::from_env();
        match backend.list() {
            Ok(targets) => Self::with_backend(targets, backend),
            Err(err) => {
                let mut app = Self::with_backend(Targets::default(), backend);
                app.set_persistent_status(format!("Load failed: {err}"));
                app
            }
        }
    }

    #[cfg(test)]
    pub fn with_targets(targets: Targets) -> Self {
        Self::with_backend(targets.clone(), shell::Backend::virtual_targets(targets))
    }

    fn with_backend(targets: Targets, backend: shell::Backend) -> Self {
        let mut app = Self::empty();
        app.targets = targets;
        app.backend = backend;
        app.refresh_order();
        app
    }

    pub fn is_virtual(&self) -> bool {
        self.backend.is_virtual()
    }

    pub fn refresh_order(&mut self) {
        self.target_order = self.targets.keys().cloned().collect();
        self.target_order.sort();

        if self.target_order.is_empty() {
            self.selected_target = 0;
            self.selected_overlay = 0;
            return;
        }

        if self.selected_target >= self.target_order.len() {
            self.selected_target = self.target_order.len() - 1;
        }

        let max_overlay = self.actionable_overlays().len().saturating_sub(1);
        self.selected_overlay = self.selected_overlay.min(max_overlay);
    }

    pub fn current_target_name(&self) -> Option<&str> {
        self.target_order
            .get(self.selected_target)
            .map(String::as_str)
    }

    pub fn current_target(&self) -> Option<&TargetOverlays> {
        self.current_target_name()
            .and_then(|name| self.targets.get(name))
    }

    pub fn actionable_overlays(&self) -> Vec<OverlayRow> {
        let Some(target) = self.current_target() else {
            return Vec::new();
        };

        let mut rows = Vec::with_capacity(target.enabled.len() + target.disabled.len());
        for name in &target.enabled {
            rows.push(OverlayRow {
                name: name.clone(),
                enabled: true,
            });
        }
        for name in &target.disabled {
            rows.push(OverlayRow {
                name: name.clone(),
                enabled: false,
            });
        }
        rows
    }

    pub fn broken_overlays(&self) -> Vec<String> {
        self.current_target()
            .map(|t| t.broken.clone())
            .unwrap_or_default()
    }

    pub fn enter_detail(&mut self) {
        let max = self.actionable_overlays().len().saturating_sub(1);
        if self.selected_overlay > max {
            self.selected_overlay = 0;
        }
        self.detail_scroll = 0;
        self.clear_status();
        self.screen = Screen::Detail;
    }

    pub fn back_from_about(&mut self) {
        self.screen = self.about_prev;
    }

    pub fn move_target_up(&mut self) {
        self.page_target_up(1);
    }

    pub fn move_target_down(&mut self) {
        self.page_target_down(1);
    }

    pub fn page_target_up(&mut self, page: u16) {
        self.selected_target = self.selected_target.saturating_sub(page as usize);
        self.after_target_jump();
    }

    pub fn page_target_down(&mut self, page: u16) {
        if !self.target_order.is_empty() {
            self.selected_target =
                (self.selected_target + page as usize).min(self.target_order.len() - 1);
        }
        self.after_target_jump();
    }

    pub fn select_target_first(&mut self) {
        self.selected_target = 0;
        self.after_target_jump();
    }

    pub fn select_target_last(&mut self) {
        self.selected_target = self.target_order.len().saturating_sub(1);
        self.after_target_jump();
    }

    /// The overlay cursor and detail scroll belong to the previous target.
    fn after_target_jump(&mut self) {
        self.selected_overlay = 0;
        self.detail_scroll = 0;
    }

    pub fn move_overlay_up(&mut self) {
        self.page_overlay_up(1);
    }

    pub fn move_overlay_down(&mut self) {
        self.page_overlay_down(1);
    }

    pub fn page_overlay_up(&mut self, page: u16) {
        self.selected_overlay = self.selected_overlay.saturating_sub(page as usize);
    }

    pub fn page_overlay_down(&mut self, page: u16) {
        let max = self.actionable_overlays().len().saturating_sub(1);
        self.selected_overlay = (self.selected_overlay + page as usize).min(max);
    }

    pub fn select_overlay_first(&mut self) {
        self.selected_overlay = 0;
    }

    pub fn select_overlay_last(&mut self) {
        self.selected_overlay = self.actionable_overlays().len().saturating_sub(1);
    }

    pub fn apply_selected_overlay(&mut self) -> io::Result<()> {
        let target_name = match self.current_target_name() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        let rows = self.actionable_overlays();
        let Some(row) = rows.get(self.selected_overlay) else {
            self.set_status("No overlay selected.");
            return Ok(());
        };

        let new_enabled = !row.enabled;
        let overlay_name = row.name.clone();

        // Surface shell errors in the status bar instead of crashing.
        if let Err(e) = self.backend.set_overlay(new_enabled, &overlay_name) {
            self.set_status(format!("Error: {e}"));
            return Ok(());
        }

        self.set_status(if new_enabled {
            format!("✓ Enabled {overlay_name}")
        } else {
            format!("○ Disabled {overlay_name}")
        });

        if let Err(e) = self.reload_keep_focus(Some(target_name), Some(overlay_name)) {
            self.set_persistent_status(format!("Reload failed: {e}"));
        }

        Ok(())
    }

    pub fn reload_keep_focus(
        &mut self,
        keep_target: Option<String>,
        keep_overlay: Option<String>,
    ) -> io::Result<()> {
        self.targets = self.backend.list()?;
        self.refresh_order();

        if let Some(idx) = keep_target
            .as_ref()
            .and_then(|target| self.target_order.iter().position(|n| n == target))
        {
            self.selected_target = idx;
        }

        if let Some(ref overlay_name) = keep_overlay {
            let rows = self.actionable_overlays();
            if let Some(idx) = rows.iter().position(|r| r.name == *overlay_name) {
                self.selected_overlay = idx;
            }
        }

        Ok(())
    }

    pub fn selected_overlay_line(&self) -> u16 {
        let actionable = self.actionable_overlays();
        let enabled_count = actionable.iter().filter(|r| r.enabled).count();

        if self.selected_overlay < enabled_count {
            1u16 + self.selected_overlay as u16
        } else {
            let offset: u16 = if enabled_count > 0 {
                1 + enabled_count as u16 + 1 + 1
            } else {
                1
            };
            let row_index = (self.selected_overlay - enabled_count) as u16;
            offset + row_index
        }
    }

    pub fn clamp_detail_scroll(&mut self, viewport_height: u16) {
        if viewport_height == 0 {
            return;
        }
        let line = self.selected_overlay_line();
        if line >= self.detail_scroll.saturating_add(viewport_height) {
            self.detail_scroll = line + 1 - viewport_height;
        }
        if line < self.detail_scroll {
            self.detail_scroll = line;
        }
    }

    pub fn tick(&mut self) {
        if self.status.is_empty() || self.status_persistent {
            self.status_expires_at = None;
        } else {
            match self.status_expires_at {
                Some(expires_at) => {
                    if Instant::now() >= expires_at {
                        self.status.clear();
                        self.status_expires_at = None;
                    }
                }
                None => {
                    self.status_expires_at = Some(Instant::now() + Duration::from_secs(3));
                }
            }
        }
    }

    pub fn enter_about(&mut self) {
        self.about_scroll = 0;
        self.screen = Screen::About;
    }

    pub fn scroll_about_up(&mut self) {
        self.about_scroll = self.about_scroll.saturating_sub(1);
    }

    pub fn scroll_about_to_top(&mut self) {
        self.about_scroll = 0;
    }

    pub fn scroll_about_down(&mut self, max_scroll: u16) {
        if self.about_scroll < max_scroll {
            self.about_scroll = self.about_scroll.saturating_add(1);
        }
    }

    pub fn scroll_about_page_up(&mut self, amount: u16) {
        self.about_scroll = self.about_scroll.saturating_sub(amount);
    }

    pub fn scroll_about_page_down(&mut self, amount: u16, max_scroll: u16) {
        self.about_scroll = (self.about_scroll + amount).min(max_scroll);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_targets() -> Targets {
        let enabled = TargetOverlays {
            enabled: vec!["a.enabled1".into(), "a.enabled2".into()],
            disabled: vec!["a.disabled1".into(), "a.disabled2".into()],
            broken: vec!["a.broken1".into()],
        };

        let second = TargetOverlays {
            disabled: vec!["b.disabled1".into()],
            ..TargetOverlays::default()
        };

        Targets::from([
            ("com.acme.a".to_string(), enabled),
            ("com.acme.b".to_string(), second),
        ])
    }

    #[test]
    fn refresh_order_sorts_and_clamps_selection() {
        let mut app = App::with_targets(fixture_targets());
        assert_eq!(app.target_order, ["com.acme.a", "com.acme.b"]);

        app.selected_target = 99;
        app.selected_overlay = 99;
        app.refresh_order();
        assert_eq!(app.selected_target, 1);
        assert_eq!(app.selected_overlay, 0);
    }

    #[test]
    fn actionable_overlays_excludes_broken() {
        let app = App::with_targets(fixture_targets());
        let rows = app.actionable_overlays();
        assert_eq!(rows.len(), 4, "broken overlays must not be toggleable");
        assert!(app.broken_overlays() == ["a.broken1"]);
    }

    #[test]
    fn selected_overlay_line_matches_rendered_layout() {
        let app = App::with_targets(fixture_targets());

        assert_eq!(app.selected_overlay, 0);
        assert_eq!(
            app.selected_overlay_line(),
            1,
            "first row sits below the section header"
        );

        let mut app = App::with_targets(fixture_targets());
        app.selected_overlay = 2;
        assert_eq!(
            app.selected_overlay_line(),
            5,
            "header + 2 enabled rows + blank + disabled header"
        );
    }

    #[test]
    fn clamp_detail_scroll_follows_selection() {
        let mut app = App::with_targets(fixture_targets());
        app.selected_overlay = 2;

        app.clamp_detail_scroll(3);
        assert_eq!(app.detail_scroll, 3, "line 5 is last visible (5 + 1 - 3)");

        app.clamp_detail_scroll(10);
        assert_eq!(app.detail_scroll, 3, "line 5 already visible");

        app.selected_overlay = 0;
        app.clamp_detail_scroll(10);
        assert_eq!(app.detail_scroll, 1, "scrolls back to the selected line");
    }

    #[test]
    fn load_failure_status_persists() {
        let mut app = App::empty();
        app.set_persistent_status("Load failed: boom");
        app.tick();
        app.tick();
        assert_eq!(app.status, "Load failed: boom");

        app.set_status("transient");
        app.tick();
        assert_eq!(app.status, "transient");
        app.status_expires_at = Some(Instant::now() - Duration::from_secs(1));
        app.tick();
        assert!(app.status.is_empty());
    }

    #[test]
    fn target_jumps_clamp_and_reset_overlay_cursor() {
        let mut app = App::with_targets(fixture_targets());
        app.selected_overlay = 1;

        app.select_target_last();
        assert_eq!(app.selected_target, 1);
        assert_eq!(app.selected_overlay, 0, "overlay cursor must reset");

        app.select_target_first();
        app.page_target_down(50);
        assert_eq!(app.selected_target, 1, "oversized page clamps to the end");

        app.page_target_up(50);
        assert_eq!(app.selected_target, 0, "page-up saturates at the top");

        let mut empty = App::empty();
        empty.select_target_last();
        empty.page_target_down(5);
        assert_eq!(empty.selected_target, 0, "empty list stays at 0");
    }

    #[test]
    fn overlay_jumps_clamp_to_actionable_rows() {
        let mut app = App::with_targets(fixture_targets());

        app.select_overlay_last();
        assert_eq!(app.selected_overlay, 3, "4 actionable rows, 0-based");

        app.page_overlay_down(50);
        assert_eq!(app.selected_overlay, 3, "never past the last row");

        app.page_overlay_up(50);
        assert_eq!(app.selected_overlay, 0);

        app.selected_overlay = 99;
        app.move_overlay_down();
        assert_eq!(app.selected_overlay, 3, "stale cursor is reeled back in");
    }
}
