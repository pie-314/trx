//! Status-bar widget API.
//!
//! Each widget implements [`StatusWidget`].  The registry lives in [`App`] and
//! is built once from `config.settings.status_widgets`.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// A small, self-contained piece of UI rendered inside the status bar.
///
/// The contract is intentionally minimal:
/// - [`tick`] is called once per render frame (~10 Hz) so widgets can refresh
///   their internal state without blocking the event loop.
/// - [`render`] draws into the area allocated by `draw_status_bar`.
/// - [`name`] is the key used in `config.status_widgets`.
/// - [`min_width`] lets the layout engine reserve the right amount of space.
pub trait StatusWidget: Send + Sync {
    /// Refresh internal state (non-blocking).
    fn tick(&mut self);

    /// Draw into the given area.
    fn render(&self, frame: &mut Frame, area: Rect);

    /// The config key that activates this widget (e.g. `"clock"`).
    /// Used by the registry builder to match `config.status_widgets` entries.
    #[allow(dead_code)]
    fn name(&self) -> &'static str;

    /// Minimum character width needed to display this widget without clipping.
    fn min_width(&self) -> u16;
}

// ---------------------------------------------------------------------------
// ClockWidget — current local time HH:MM:SS
// ---------------------------------------------------------------------------

/// Displays the current local time as `HH:MM:SS`.
///
/// Uses only `std::time` — no external dependencies.
pub struct ClockWidget {
    /// Cached time string, refreshed on every `tick()`.
    display: String,
}

impl ClockWidget {
    pub fn new() -> Self {
        let mut w = Self { display: String::new() };
        w.tick();
        w
    }

    /// Convert seconds-since-midnight → `HH:MM:SS`.
    fn format_time(secs_since_epoch: u64) -> String {
        // UTC offset is ignored intentionally — we use UTC for simplicity
        // (avoids the tz dependency).  A future improvement could add local tz.
        let secs_in_day = secs_since_epoch % 86_400;
        let h = secs_in_day / 3600;
        let m = (secs_in_day % 3600) / 60;
        let s = secs_in_day % 60;
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

impl StatusWidget for ClockWidget {
    fn tick(&mut self) {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        self.display = format!(" 🕐 {} ", Self::format_time(secs));
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let p = Paragraph::new(Line::from(Span::styled(
            &self.display,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(p, area);
    }

    fn name(&self) -> &'static str {
        "clock"
    }

    fn min_width(&self) -> u16 {
        // " 🕐 HH:MM:SS " — clock emoji may be 2 cols wide in some terminals
        14
    }
}

// ---------------------------------------------------------------------------
// CpuWidget — CPU usage from /proc/stat (Linux); graceful no-op elsewhere
// ---------------------------------------------------------------------------

/// Displays overall CPU utilisation as a percentage.
///
/// On Linux the value is derived by sampling `/proc/stat` on every `tick()`.
/// On all other platforms the widget renders `cpu:n/a`.
pub struct CpuWidget {
    /// Most-recently computed usage string (e.g. `"cpu: 12%"`).
    display: String,
    /// Previous `/proc/stat` totals used to compute delta.
    #[cfg(target_os = "linux")]
    prev_idle: u64,
    #[cfg(target_os = "linux")]
    prev_total: u64,
}

impl CpuWidget {
    pub fn new() -> Self {
        Self {
            display: " cpu:-- ".to_string(),
            #[cfg(target_os = "linux")]
            prev_idle: 0,
            #[cfg(target_os = "linux")]
            prev_total: 0,
        }
    }

    /// Parse the first line of `/proc/stat` and return `(idle, total)`.
    #[cfg(target_os = "linux")]
    fn read_proc_stat() -> Option<(u64, u64)> {
        use std::fs;
        let content = fs::read_to_string("/proc/stat").ok()?;
        let first_line = content.lines().next()?;
        // Format: "cpu  user nice system idle iowait irq softirq steal guest guest_nice"
        let mut parts = first_line.split_whitespace();
        parts.next(); // skip "cpu"
        let values: Vec<u64> = parts.filter_map(|v| v.parse().ok()).collect();
        if values.len() < 4 {
            return None;
        }
        let idle = values[3]; // idle time
        let total: u64 = values.iter().sum();
        Some((idle, total))
    }
}

impl StatusWidget for CpuWidget {
    fn tick(&mut self) {
        #[cfg(target_os = "linux")]
        {
            if let Some((idle, total)) = Self::read_proc_stat() {
                let d_idle = idle.saturating_sub(self.prev_idle);
                let d_total = total.saturating_sub(self.prev_total);
                self.prev_idle = idle;
                self.prev_total = total;
                if d_total > 0 {
                    let usage = 100u64.saturating_sub(d_idle * 100 / d_total);
                    self.display = format!(" cpu:{:2}% ", usage);
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.display = " cpu:n/a ".to_string();
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let color = if self.display.contains("n/a") || self.display.contains("--") {
            Color::DarkGray
        } else {
            // Parse usage % for colour-coding: green < 50, yellow < 80, red ≥ 80
            let pct: u64 = self
                .display
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            if pct >= 80 {
                Color::Red
            } else if pct >= 50 {
                Color::Yellow
            } else {
                Color::Green
            }
        };

        let p = Paragraph::new(Line::from(Span::styled(
            &self.display,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(p, area);
    }

    fn name(&self) -> &'static str {
        "cpu"
    }

    fn min_width(&self) -> u16 {
        // " cpu:100% " = 10 chars
        10
    }
}

// ---------------------------------------------------------------------------
// ManagerWidget — active package manager name
// ---------------------------------------------------------------------------

/// Displays the name of the currently active package manager.
pub struct ManagerWidget {
    manager_name: String,
}

impl ManagerWidget {
    pub fn new(name: String) -> Self {
        Self { manager_name: name }
    }

    /// Update the displayed manager name (call when the user switches manager).
    #[allow(dead_code)]
    pub fn set_name(&mut self, name: String) {
        self.manager_name = name;
    }
}

impl StatusWidget for ManagerWidget {
    fn tick(&mut self) {
        // Manager name is set externally via set_name(); nothing to poll.
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!(" 📦 {} ", self.manager_name);
        let p = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(p, area);
    }

    fn name(&self) -> &'static str {
        "manager"
    }

    fn min_width(&self) -> u16 {
        // " 📦 " (3) + manager name (up to ~12) + " " = ~16
        16
    }
}

// ---------------------------------------------------------------------------
// Registry builder
// ---------------------------------------------------------------------------

/// Build the widget list from the `status_widgets` config setting.
///
/// Unknown names are silently ignored — this lets future widgets be added
/// without breaking older configs that list them.
pub fn build_widgets(
    enabled: &[String],
    manager_name: String,
) -> Vec<Box<dyn StatusWidget>> {
    enabled
        .iter()
        .filter_map(|key| -> Option<Box<dyn StatusWidget>> {
            match key.as_str() {
                "clock" => Some(Box::new(ClockWidget::new())),
                "cpu" => Some(Box::new(CpuWidget::new())),
                "manager" => Some(Box::new(ManagerWidget::new(manager_name.clone()))),
                _ => None, // unknown widget key — ignore gracefully
            }
        })
        .collect()
}
