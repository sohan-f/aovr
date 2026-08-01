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
    pub targets: Targets,
    pub target_order: Vec<String>,
    pub selected_target: usize,
    pub selected_overlay: usize,
    pub about_prev: Screen,
    pub status: String,
    pub status_expires_at: Option<Instant>,
    pub detail_scroll: u16,
    pub about_scroll: u16,
}

impl App {
    pub fn empty() -> Self {
        Self {
            screen: Screen::Targets,
            targets: Targets::default(),
            target_order: Vec::new(),
            selected_target: 0,
            selected_overlay: 0,
            about_prev: Screen::Targets,
            status: String::new(),
            detail_scroll: 0,
            status_expires_at: None,
            about_scroll: 0,
        }
    }

    pub fn load() -> io::Result<Self> {
        let targets = shell::load_targets()?;
        Ok(Self::with_targets(targets))
    }

    pub fn with_targets(targets: Targets) -> Self {
        let mut app = Self::empty();
        app.targets = targets;
        app.refresh_order();
        app
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
        self.status.clear();
        self.screen = Screen::Detail;
    }

    pub fn back_from_about(&mut self) {
        self.screen = self.about_prev;
    }

    pub fn move_target_up(&mut self) {
        if self.selected_target > 0 {
            self.selected_target -= 1;
            self.selected_overlay = 0;
            self.detail_scroll = 0;
        }
    }

    pub fn move_target_down(&mut self) {
        if self.selected_target + 1 < self.target_order.len() {
            self.selected_target += 1;
            self.selected_overlay = 0;
            self.detail_scroll = 0;
        }
    }

    pub fn move_overlay_up(&mut self) {
        self.selected_overlay = self.selected_overlay.saturating_sub(1);
    }

    pub fn move_overlay_down(&mut self) {
        let max = self.actionable_overlays().len();
        if self.selected_overlay + 1 < max {
            self.selected_overlay += 1;
        }
    }

    /// Toggle the selected overlay.  Returns an error message string on failure
    /// so the caller can decide whether to surface it or propagate as io::Error.
    pub fn apply_selected_overlay(&mut self) -> io::Result<()> {
        let target_name = match self.current_target_name() {
            Some(n) => n.to_string(),
            None => return Ok(()),
        };

        let rows = self.actionable_overlays();
        let Some(row) = rows.get(self.selected_overlay) else {
            self.status = "No overlay selected.".to_string();
            return Ok(());
        };

        let new_enabled = !row.enabled;
        let overlay_name = row.name.clone();

        // Surface shell errors into the status bar instead of crashing.
        if let Err(e) = shell::set_overlay(new_enabled, &overlay_name) {
            self.status = format!("Error: {e}");
            return Ok(());
        }

        self.status = if new_enabled {
            format!("✓ Enabled {overlay_name}")
        } else {
            format!("○ Disabled {overlay_name}")
        };

        if let Err(e) = self.reload_keep_focus(Some(target_name), Some(overlay_name)) {
            self.status = format!("Reload failed: {e}");
        }

        Ok(())
    }

    pub fn reload_keep_focus(
        &mut self,
        keep_target: Option<String>,
        keep_overlay: Option<String>,
    ) -> io::Result<()> {
        self.targets = shell::load_targets()?;
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

    /// Returns the 0-based line number of the selected overlay within the
    /// rendered Detail paragraph (accounting for section headers and blanks).
    ///
    /// Layout (all 0-based lines):
    ///   [0]         ✓ Enabled  header          (only if enabled_count > 0)
    ///   [1..N]      enabled overlay rows
    ///   [N+1]       blank separator             (only if both sections exist)
    ///   [N+2]       ○ Disabled header           (only if disabled_count > 0)
    ///   [N+3..]     disabled overlay rows
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
        if self.status.is_empty() {
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
