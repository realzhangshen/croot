use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use crate::input::handler::Action;

use super::colors;

/// A single toolbar button.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolbarButton {
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub action: Action,
    /// Computed column range during render (start inclusive, end exclusive).
    pub x_start: u16,
    pub x_end: u16,
}

/// State for the toolbar row.
#[derive(Debug)]
pub struct ToolbarState {
    pub buttons: Vec<ToolbarButton>,
    pub project_name: String,
}

impl ToolbarState {
    pub fn new(project_name: &str) -> Self {
        let buttons = vec![
            ToolbarButton {
                icon: " + ",
                tooltip: "New File",
                action: Action::NewFile,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " +/ ",
                tooltip: "New Directory",
                action: Action::NewDir,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " \u{21bb} ",
                tooltip: "Refresh",
                action: Action::Refresh,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " \u{229f} ",
                tooltip: "Collapse All",
                action: Action::CollapseAll,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " / ",
                tooltip: "Search",
                action: Action::StartSearch,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " \u{229e} ",
                tooltip: "Toggle Preview",
                action: Action::TogglePreview,
                x_start: 0,
                x_end: 0,
            },
            ToolbarButton {
                icon: " \u{00d7} ",
                tooltip: "Quit",
                action: Action::Quit,
                x_start: 0,
                x_end: 0,
            },
        ];

        Self {
            buttons,
            project_name: project_name.to_string(),
        }
    }

    /// Map a click column to a toolbar button action.
    pub fn button_at(&self, col: u16) -> Option<Action> {
        self.buttons
            .iter()
            .find(|b| col >= b.x_start && col < b.x_end)
            .map(|b| b.action.clone())
    }
}

pub struct ToolbarWidget<'a> {
    pub state: &'a mut ToolbarState,
    pub hover_col: Option<u16>,
}

impl Widget for ToolbarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let base_style = Style::default().add_modifier(Modifier::REVERSED);
        let name_style = Style::default()
            .add_modifier(Modifier::REVERSED | Modifier::BOLD)
            .fg(Color::Cyan);
        let hover_style = colors::popup_selected();

        // Fill background
        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut((x, area.y)) {
                cell.set_style(base_style);
                cell.set_symbol(" ");
            }
        }

        // Render project name on the left
        let name_display = format!(" {} ", self.state.project_name);
        let name_width = name_display.width() as u16;
        buf.set_string(area.x, area.y, &name_display, name_style);

        // Render buttons right-aligned
        let total_button_width: u16 = self
            .state
            .buttons
            .iter()
            .map(|b| b.icon.width() as u16)
            .sum();

        let buttons_start = area
            .x
            .saturating_add(area.width)
            .saturating_sub(total_button_width);

        // Don't render buttons if they'd overlap with name; zero out hitboxes
        if buttons_start <= area.x + name_width {
            for btn in &mut self.state.buttons {
                btn.x_start = 0;
                btn.x_end = 0;
            }
            return;
        }

        let mut col = buttons_start;
        for btn in &mut self.state.buttons {
            let w = btn.icon.width() as u16;
            btn.x_start = col;
            btn.x_end = col + w;

            let is_hover = self
                .hover_col
                .is_some_and(|hc| hc >= btn.x_start && hc < btn.x_end);

            let style = if is_hover { hover_style } else { base_style };

            // Fill button area with style first (for hover highlight)
            if is_hover {
                for x in btn.x_start..btn.x_end {
                    if let Some(cell) = buf.cell_mut((x, area.y)) {
                        cell.set_style(style);
                        cell.set_symbol(" ");
                    }
                }
            }

            buf.set_string(col, area.y, btn.icon, style);
            col += w;
        }
    }
}
