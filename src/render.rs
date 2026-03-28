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
    layout::Rect,
    prelude::Widget,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

use crate::{BindingGroup, Key, WhichKey, WhichKeyState};

/// Render the which-key popup.
pub fn render_popup<K, S, A, C>(
    config: &WhichKey,
    buf: &mut Buffer,
    state: &WhichKeyState<K, S, A, C>,
) where
    K: Key + Clone + PartialEq,
    S: Clone + Ord + PartialEq + Send + Sync,
    A: Clone + Send + Sync,
    C: Clone + std::fmt::Display,
{
    let groups = state.current_bindings();
    if groups.is_empty() {
        return;
    }

    let columns = build_columns(&groups, config.max_height);

    let title = if state.current_sequence.is_empty() {
        " Shortcuts ".to_string()
    } else {
        format!(" {} ", state.format_path())
    };

    let popup_area = calculate_popup_area(config, *buf.area(), &columns, &title);

    // Clear the area first
    Clear.render(popup_area, buf);

    let block = Block::default()
        .title(title.as_str())
        .borders(Borders::ALL)
        .border_style(config.border_style)
        .padding(Padding::horizontal(1));

    let inner_area = block.inner(popup_area);
    block.render(popup_area, buf);

    // Render columns
    let column_areas = layout_columns(&columns, inner_area);
    for (col_area, col_data) in column_areas.iter().zip(columns.iter()) {
        render_column(buf, *col_area, col_data, config);
    }
}

/// Column data for rendering.
#[derive(Debug, Clone)]
struct ColumnData<K: Key> {
    groups: Vec<(String, Vec<(K, String)>)>,
    max_key_width: usize,
    max_desc_width: usize,
}

impl<K: Key> ColumnData<K> {
    const fn content_width(&self) -> usize {
        self.max_key_width + 1 + self.max_desc_width
    }
}

/// Build column data from binding groups.
fn build_columns<K: Key>(groups: &[BindingGroup<K>], max_height: u16) -> Vec<ColumnData<K>> {
    let rows_per_column = max_height.saturating_sub(2) as usize;
    let mut columns: Vec<ColumnData<K>> = Vec::new();
    let mut current_groups: Vec<(String, Vec<(K, String)>)> = Vec::new();
    let mut current_rows = 0usize;

    for group in groups {
        let items: Vec<(K, String)> = group
            .bindings
            .iter()
            .map(|b| (b.key.clone(), b.description.clone()))
            .collect();
        let group_rows = items.len() + 1; // +1 for category header

        if current_rows + group_rows > rows_per_column && current_rows > 0 {
            columns.push(build_column_data(current_groups));
            current_groups = Vec::new();
            current_rows = 0;
        }

        current_groups.push((group.category.clone(), items));
        current_rows += group_rows;
    }

    if !current_groups.is_empty() {
        columns.push(build_column_data(current_groups));
    }

    columns
}

fn build_column_data<K: Key>(groups: Vec<(String, Vec<(K, String)>)>) -> ColumnData<K> {
    let max_key_width = groups
        .iter()
        .flat_map(|(_, items)| items.iter())
        .map(|(k, _)| k.display().len())
        .max()
        .unwrap_or(5);

    let max_desc_width = groups
        .iter()
        .flat_map(|(_, items)| items.iter())
        .map(|(_, d)| d.len())
        .max()
        .unwrap_or(10);

    ColumnData {
        groups,
        max_key_width,
        max_desc_width,
    }
}

/// Calculate the popup area based on position and content.
#[allow(clippy::cast_possible_truncation)]
fn calculate_popup_area<K: Key>(
    config: &WhichKey,
    frame_area: Rect,
    columns: &[ColumnData<K>],
    title: &str,
) -> Rect {
    let column_gap = 1u16;
    let total_content_width: u16 = columns.iter().map(|c| c.content_width() as u16).sum();
    let total_gap = column_gap * columns.len().saturating_sub(1) as u16;

    let title_width = title.len() as u16 + 2;
    let min_width = total_content_width + total_gap + 4;
    let popup_width = min_width
        .max(title_width)
        .min(frame_area.width.saturating_sub(2));
    let popup_height = config.max_height.min(frame_area.height.saturating_sub(2));

    let x = match config.position {
        crate::PopupPosition::BottomLeft | crate::PopupPosition::TopLeft => 1,
        crate::PopupPosition::BottomRight | crate::PopupPosition::TopRight => frame_area
            .width
            .saturating_sub(popup_width)
            .saturating_sub(1),
    };

    let y = match config.position {
        crate::PopupPosition::BottomLeft | crate::PopupPosition::BottomRight => frame_area
            .height
            .saturating_sub(popup_height)
            .saturating_sub(1),
        crate::PopupPosition::TopLeft | crate::PopupPosition::TopRight => 1,
    };

    Rect::new(x, y, popup_width, popup_height)
}

/// Layout columns within the inner area.
#[allow(clippy::cast_possible_truncation)]
fn layout_columns<K: Key>(columns: &[ColumnData<K>], inner_area: Rect) -> Vec<Rect> {
    let column_gap = 1u16;
    let mut result = Vec::with_capacity(columns.len());
    let mut x = inner_area.x;

    for column_data in columns {
        let width = column_data.content_width() as u16;
        result.push(Rect::new(x, inner_area.y, width, inner_area.height));
        x += width + column_gap;
    }

    result
}

/// Render a single column.
fn render_column<K: Key>(
    buf: &mut Buffer,
    area: Rect,
    column_data: &ColumnData<K>,
    config: &WhichKey,
) {
    let mut y = area.y;

    for (category, items) in &column_data.groups {
        if y >= area.bottom() {
            break;
        }

        // Render category header (if not empty)
        if !category.is_empty() {
            let header = Paragraph::new(category.clone()).style(config.category_style);
            header.render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }

        // Render bindings
        for (key, description) in items {
            if y >= area.bottom() {
                break;
            }

            let key_display = key.display();
            let key_span = Span::styled(
                format!("{:>width$}", key_display, width = column_data.max_key_width),
                config.key_style,
            );
            let desc_span = Span::styled(format!(" {description}"), config.description_style);
            let line = Line::from(vec![key_span, desc_span]);
            let para = Paragraph::new(line);
            para.render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }
    }
}
