// Copyright (C) 2026 Jayson Lennon
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program; if not, see <https://opensource.org/license/lgpl-3-0>.

use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};

use crate::{Key, WhichKeyState, render};

/// Position of the which-key popup on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupPosition {
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-right corner (default).
    #[default]
    BottomRight,
    /// Top-left corner.
    TopLeft,
    /// Top-right corner.
    TopRight,
}

/// Configuration for the which-key widget.
///
/// This struct holds styling and positioning configuration.
/// The actual state is held in `WhichKeyState`.
#[derive(Debug, Clone)]
pub struct WhichKey {
    /// Maximum height of the popup.
    pub max_height: u16,
    /// Position of the popup.
    pub position: PopupPosition,
    /// Border style.
    pub border_style: Style,
    /// Key text style.
    pub key_style: Style,
    /// Description text style.
    pub description_style: Style,
    /// Category header style.
    pub category_style: Style,
}

impl Default for WhichKey {
    fn default() -> Self {
        Self {
            max_height: 10,
            position: PopupPosition::default(),
            border_style: Style::default().fg(Color::Yellow),
            key_style: Style::default().fg(Color::Cyan),
            description_style: Style::default(),
            category_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl WhichKey {
    /// Create a new which-key widget with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum height.
    #[must_use]
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Set the popup position.
    #[must_use]
    pub fn position(mut self, position: PopupPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }
}

impl WhichKey {
    /// Render the which-key popup.
    ///
    /// This method renders the widget with the given state.
    /// If the state is not active, nothing is rendered.
    pub fn render<K, S, A, C>(self, buf: &mut Buffer, state: &WhichKeyState<K, S, A, C>)
    where
        K: Key + Clone + PartialEq,
        S: Clone + Ord + PartialEq + Send + Sync,
        A: Clone + Send + Sync,
        C: Clone + std::fmt::Display,
    {
        if !state.active && state.current_sequence.is_empty() {
            return;
        }

        render::render_popup(&self, buf, state);
    }
}
