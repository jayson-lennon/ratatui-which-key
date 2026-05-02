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

use crate::{BindingGroup, DisplayMode, Key, LayoutStrategy, WhichKey, WhichKeyState};

/// Minimum whitespace between columns for Flat+Wide layout.
const FLAT_WIDE_COLUMN_GAP: usize = 6;
/// Maximum column width for Flat+Wide layout.
const FLAT_WIDE_MAX_COL_WIDTH: usize = 30;
/// Minimum number of columns for Flat+Wide layout.
const FLAT_WIDE_MIN_COLS: usize = 3;
/// Maximum number of columns for Flat+Wide layout.
const FLAT_WIDE_MAX_COLS: usize = 5;

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

    let columns = build_columns(&groups, config, buf.area().height, buf.area().width);

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
    let column_gap = flat_wide_column_gap(config);
    let column_areas = layout_columns(&columns, inner_area, column_gap);
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
    /// Fixed column width for Flat+Wide. 0 = auto (compute from content).
    target_column_width: usize,
}

impl<K: Key> ColumnData<K> {
    const fn content_width(&self) -> usize {
        if self.target_column_width > 0 {
            self.target_column_width
        } else {
            self.max_key_width + 1 + self.max_desc_width
        }
    }

    fn row_count(&self) -> usize {
        self.groups
            .iter()
            .map(|(category, items)| {
                let header = usize::from(!category.is_empty());
                header + items.len()
            })
            .sum()
    }
}

/// Collect all bindings across groups into a single sorted list for Flat mode.
fn flatten_bindings<K: Key>(groups: &[BindingGroup<K>]) -> Vec<(K, String)> {
    let mut items: Vec<(K, String)> = groups
        .iter()
        .flat_map(|g| g.bindings.iter())
        .map(|b| (b.key.clone(), b.description.clone()))
        .collect();
    items.sort_by(|a, b| a.0.display().cmp(&b.0.display()));
    items
}

/// Build column data from binding groups using auto-sizing.
///
/// The layout depends on the combination of `DisplayMode` and `LayoutStrategy`:
/// - **Category + PreferTall**: single column, split into additional columns evenly when content exceeds available height
/// - **Category + PreferWide**: each category gets its own column, split only if a single category overflows
/// - **Flat + PreferTall**: all bindings sorted alphabetically, single column, split evenly on overflow
/// - **Flat + PreferWide**: fixed column width, distribute bindings evenly across columns that fit in terminal width
#[allow(clippy::cast_possible_truncation)]
fn build_columns<K: Key>(
    groups: &[BindingGroup<K>],
    config: &WhichKey,
    frame_height: u16,
    frame_width: u16,
) -> Vec<ColumnData<K>> {
    let available_height = frame_height.saturating_sub(2) as usize; // 2 for borders

    match (config.display_mode, config.layout_strategy) {
        (DisplayMode::Flat, LayoutStrategy::PreferTall) => {
            build_flat_tall(groups, available_height)
        }
        (DisplayMode::Flat, LayoutStrategy::PreferWide) => {
            build_flat_wide(groups, frame_width)
        }
        (DisplayMode::Category, LayoutStrategy::PreferTall) => {
            build_category_tall(groups, available_height)
        }
        (DisplayMode::Category, LayoutStrategy::PreferWide) => {
            build_category_wide(groups, available_height)
        }
    }
}

/// Flat + PreferTall: single column of sorted bindings, split evenly when content overflows.
fn build_flat_tall<K: Key>(
    groups: &[BindingGroup<K>],
    available_height: usize,
) -> Vec<ColumnData<K>> {
    let items = flatten_bindings(groups);
    if items.is_empty() {
        return Vec::new();
    }

    let total_rows = items.len(); // flat mode: no category headers

    if total_rows <= available_height {
        // Fits in a single column
        return vec![build_column_data(vec![(String::new(), items)])];
    }

    // Split into multiple columns, distributing evenly
    let num_columns = total_rows.div_ceil(available_height);
    distribute_flat_into_columns(items, num_columns)
}

/// Flat + PreferWide: compute column count (3–5) and width (max 30) from terminal width with 6-space gaps.
fn build_flat_wide<K: Key>(
    groups: &[BindingGroup<K>],
    frame_width: u16,
) -> Vec<ColumnData<K>> {
    let items = flatten_bindings(groups);
    if items.is_empty() {
        return Vec::new();
    }

    let available = frame_width.saturating_sub(4) as usize;

    // Try to fit as many columns as possible at max width.
    let (num_columns, col_width) = (FLAT_WIDE_MIN_COLS..=FLAT_WIDE_MAX_COLS)
        .rev()
        .find_map(|n| {
            let needed = n * FLAT_WIDE_MAX_COL_WIDTH + (n - 1) * FLAT_WIDE_COLUMN_GAP;
            (needed <= available).then_some((n, FLAT_WIDE_MAX_COL_WIDTH))
        })
        .unwrap_or_else(|| {
            let col_width =
                available.saturating_sub((FLAT_WIDE_MIN_COLS - 1) * FLAT_WIDE_COLUMN_GAP)
                    / FLAT_WIDE_MIN_COLS;
            (FLAT_WIDE_MIN_COLS, col_width)
        });

    let max_key_width = items
        .iter()
        .map(|(k, _)| k.display().len())
        .max()
        .unwrap_or(1);
    let max_desc_width = col_width.saturating_sub(max_key_width + 1);

    distribute_flat_wide_columns(items, num_columns, col_width, max_key_width, max_desc_width)
}

/// Distribute flat bindings across N equal-width columns for Flat+Wide mode.
fn distribute_flat_wide_columns<K: Key>(
    items: Vec<(K, String)>,
    num_columns: usize,
    col_width: usize,
    max_key_width: usize,
    max_desc_width: usize,
) -> Vec<ColumnData<K>> {
    let total = items.len();
    let base = total / num_columns;
    let extra = total % num_columns;

    let mut columns = Vec::with_capacity(num_columns);
    let mut offset = 0;

    for i in 0..num_columns {
        let count = base + usize::from(i < extra);
        let col_items = items[offset..offset + count].to_vec();
        offset += count;
        columns.push(ColumnData {
            groups: vec![(String::new(), col_items)],
            max_key_width,
            max_desc_width,
            target_column_width: col_width,
        });
    }

    columns
}

/// Distribute flat bindings evenly across N columns (left-to-right alphabetical ordering).
fn distribute_flat_into_columns<K: Key>(
    items: Vec<(K, String)>,
    num_columns: usize,
) -> Vec<ColumnData<K>> {
    let total = items.len();
    let base = total / num_columns;
    let extra = total % num_columns;

    let mut columns = Vec::with_capacity(num_columns);
    let mut offset = 0;

    for i in 0..num_columns {
        let count = base + usize::from(i < extra);
        let col_items: Vec<(K, String)> =
            items[offset..offset + count].to_vec();
        offset += count;
        columns.push(build_column_data(vec![(String::new(), col_items)]));
    }

    columns
}

/// Category + PreferTall: single column first, split into additional columns evenly when content exceeds available height.
fn build_category_tall<K: Key>(
    groups: &[BindingGroup<K>],
    available_height: usize,
) -> Vec<ColumnData<K>> {
    let group_data: Vec<(String, Vec<(K, String)>)> = groups
        .iter()
        .map(|g| {
            let items: Vec<(K, String)> = g
                .bindings
                .iter()
                .map(|b| (b.key.clone(), b.description.clone()))
                .collect();
            (g.category.clone(), items)
        })
        .collect();

    let total_rows: usize = group_data
        .iter()
        .map(|(cat, items)| {
            let header = usize::from(!cat.is_empty());
            header + items.len()
        })
        .sum();

    if total_rows <= available_height {
        // Everything fits in one column
        return vec![build_column_data(group_data)];
    }

    // Compute how many columns we need
    let num_columns = total_rows.div_ceil(available_height);
    let target_per_column = total_rows.div_ceil(num_columns);

    let mut columns = Vec::with_capacity(num_columns);
    let mut current_groups: Vec<(String, Vec<(K, String)>)> = Vec::new();
    let mut current_rows = 0usize;

    for (category, items) in group_data {
        let header = usize::from(!category.is_empty());
        let group_rows = header + items.len();

        if current_rows + group_rows > target_per_column && current_rows > 0 {
            columns.push(build_column_data(current_groups));
            current_groups = Vec::new();
            current_rows = 0;
        }

        current_groups.push((category, items));
        current_rows += group_rows;
    }

    if !current_groups.is_empty() {
        columns.push(build_column_data(current_groups));
    }

    columns
}

/// Category + PreferWide: each category gets its own column; split only if a single category overflows available height.
fn build_category_wide<K: Key>(
    groups: &[BindingGroup<K>],
    available_height: usize,
) -> Vec<ColumnData<K>> {
    let mut columns = Vec::new();

    for group in groups {
        let items: Vec<(K, String)> = group
            .bindings
            .iter()
            .map(|b| (b.key.clone(), b.description.clone()))
            .collect();

        let group_rows = items.len() + usize::from(!group.category.is_empty());

        if group_rows <= available_height {
            // Category fits in one column
            columns.push(build_column_data(vec![(
                group.category.clone(),
                items,
            )]));
        } else {
            // Category overflows — split bindings across sub-columns
            let header = usize::from(!group.category.is_empty());
            let binding_rows = items.len();
            let available_for_bindings = available_height.saturating_sub(header);
            let num_sub_cols =
                (binding_rows + available_for_bindings - 1) / available_for_bindings.max(1); // ceil
            let target_per_col = binding_rows.div_ceil(num_sub_cols);

            let mut offset = 0;
            for i in 0..num_sub_cols {
                let remaining = binding_rows - offset;
                let target = target_per_col.min(remaining);
                let sub_items: Vec<(K, String)> =
                    items[offset..offset + target].to_vec();
                offset += target;

                // First sub-column gets the category header; rest get no header
                let cat = if i == 0 {
                    group.category.clone()
                } else {
                    String::new()
                };
                columns.push(build_column_data(vec![(cat, sub_items)]));
            }
        }
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
        target_column_width: 0,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn flat_wide_column_gap(config: &WhichKey) -> u16 {
    if config.display_mode == DisplayMode::Flat
        && config.layout_strategy == LayoutStrategy::PreferWide
    {
        FLAT_WIDE_COLUMN_GAP as u16
    } else {
        1u16
    }
}

/// Truncate a description to `max_width` graphemes, appending `…` when truncated.
fn truncate_description(desc: &str, max_width: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    if max_width == 0 {
        return String::new();
    }
    let graphemes: Vec<&str> = desc.graphemes(true).collect();
    if graphemes.len() <= max_width {
        return desc.to_string();
    }
    let truncated: String = graphemes[..max_width.saturating_sub(1)]
        .iter()
        .copied()
        .collect();
    format!("{truncated}…")
}

/// Calculate the popup area based on position and content.
#[allow(clippy::cast_possible_truncation)]
fn calculate_popup_area<K: Key>(
    config: &WhichKey,
    frame_area: Rect,
    columns: &[ColumnData<K>],
    title: &str,
) -> Rect {
    let column_gap = flat_wide_column_gap(config);
    let total_content_width: u16 = columns.iter().map(|c| c.content_width() as u16).sum();
    let total_gap = column_gap * columns.len().saturating_sub(1) as u16;

    let title_width = title.len() as u16 + 2;
    let min_width = total_content_width + total_gap + 4;
    let popup_width = min_width
        .max(title_width)
        .min(frame_area.width.saturating_sub(2));
    let max_column_rows: u16 = columns
        .iter()
        .map(|c| c.row_count() as u16)
        .max()
        .unwrap_or(1);
    let popup_height = (max_column_rows + 2) // +2 for top/bottom borders
        .min(frame_area.height.saturating_sub(2));

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
fn layout_columns<K: Key>(columns: &[ColumnData<K>], inner_area: Rect, column_gap: u16) -> Vec<Rect> {
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
            let header = Paragraph::new(category.as_str()).style(config.category_style);
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
            let desc_display = truncate_description(description, column_data.max_desc_width);
            let desc_span = Span::styled(format!(" {desc_display}"), config.description_style);
            let line = Line::from(vec![key_span, desc_span]);
            let para = Paragraph::new(line);
            para.render(Rect::new(area.x, y, area.width, 1), buf);
            y += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestKey;
    use crate::Binding;

    /// Build test binding groups from simple input data.
    ///
    /// Format: `&[("category", &[("key_char", "description")])]`
    fn make_groups(groups: &[(&str, &[(&str, &str)])]) -> Vec<BindingGroup<TestKey>> {
        groups
            .iter()
            .map(|(cat, bindings)| BindingGroup {
                category: (*cat).to_string(),
                bindings: bindings
                    .iter()
                    .map(|(key, desc)| Binding {
                        key: TestKey::Char(key.chars().next().unwrap()),
                        description: (*desc).to_string(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// Count total bindings across all columns.
    fn total_bindings(columns: &[ColumnData<TestKey>]) -> usize {
        columns
            .iter()
            .flat_map(|col| col.groups.iter())
            .map(|(_, items)| items.len())
            .sum()
    }

    /// Build a config with the given display mode and layout strategy.
    fn make_config(display_mode: DisplayMode, layout_strategy: LayoutStrategy) -> WhichKey {
        WhichKey::new()
            .display_mode(display_mode)
            .layout_strategy(layout_strategy)
    }

    #[test]
    fn category_tall_fits_in_single_column() {
        // Given 3 categories with 2 bindings each (3 headers + 6 bindings = 9 rows), frame height = 20.
        let groups = make_groups(&[
            ("Nav", &[("h", "left"), ("l", "right")]),
            ("Edit", &[("d", "delete"), ("y", "yank")]),
            ("File", &[("w", "write"), ("q", "quit")]),
        ]);
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 80);

        // Then result is 1 column with all 3 groups.
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].groups.len(), 3);
        assert_eq!(columns[0].row_count(), 9);
    }

    #[test]
    fn category_tall_splits_into_multiple_columns_on_overflow() {
        // Given 3 categories with 5 bindings each (3 headers + 15 bindings = 18 rows), frame height = 10.
        let groups = make_groups(&[
            ("Nav", &[("h", "left"), ("j", "down"), ("k", "up"), ("l", "right"), ("g", "top")]),
            ("Edit", &[("d", "delete"), ("y", "yank"), ("p", "paste"), ("u", "undo"), ("r", "redo")]),
            ("File", &[("w", "write"), ("q", "quit"), ("e", "edit"), ("n", "new"), ("o", "open")]),
        ]);
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 10, 80);

        // Then result has multiple columns and no bindings are dropped.
        assert!(columns.len() >= 2, "Expected at least 2 columns, got {}", columns.len());
        assert_eq!(total_bindings(&columns), 15);
    }

    #[test]
    fn category_wide_each_category_gets_own_column() {
        // Given 3 categories with varying binding counts, frame height = 20.
        let groups = make_groups(&[
            ("Nav", &[("h", "left"), ("j", "down"), ("k", "up")]),
            ("Edit", &[("d", "delete"), ("y", "yank"), ("p", "paste"), ("u", "undo"), ("r", "redo")]),
            ("File", &[("w", "write"), ("q", "quit")]),
        ]);
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferWide);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 80);

        // Then each category gets its own column.
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].groups.len(), 1);
        assert_eq!(columns[1].groups.len(), 1);
        assert_eq!(columns[2].groups.len(), 1);

        // And each column has the correct category name.
        assert_eq!(columns[0].groups[0].0, "Nav");
        assert_eq!(columns[1].groups[0].0, "Edit");
        assert_eq!(columns[2].groups[0].0, "File");
    }

    #[test]
    fn category_wide_splits_overflowing_category_into_sub_columns() {
        // Given 1 category with 20 bindings, frame height = 10 (available = 8).
        let groups = vec![BindingGroup {
            category: "Big".to_string(),
            bindings: (0..20)
                .map(|i| Binding {
                    key: TestKey::Char(char::from(b'a' + ((i % 26) as u8))),
                    description: format!("action{i}"),
                })
                .collect(),
        }];
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferWide);

        // When building columns.
        let columns = build_columns(&groups, &config, 10, 200);

        // Then result has at least 3 sub-columns (20 bindings / 7 available_for_bindings ≈ 3).
        assert!(
            columns.len() >= 3,
            "Expected at least 3 sub-columns, got {}",
            columns.len()
        );

        // And first sub-column has the category header.
        assert_eq!(columns[0].groups[0].0, "Big");

        // And remaining sub-columns have empty headers.
        for col in &columns[1..] {
            assert!(
                col.groups[0].0.is_empty(),
                "Expected empty header, got '{}'",
                col.groups[0].0
            );
        }

        // And no bindings are dropped.
        assert_eq!(total_bindings(&columns), 20);
    }

    #[test]
    fn flat_tall_fits_in_single_column_when_content_fits() {
        // Given 3 categories with 2 bindings each (6 total), frame height = 20.
        let groups = make_groups(&[
            ("Nav", &[("c", "left"), ("d", "right")]),
            ("Edit", &[("a", "delete"), ("f", "yank")]),
            ("File", &[("b", "write"), ("e", "quit")]),
        ]);
        let config = make_config(DisplayMode::Flat, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 80);

        // Then result is 1 column with 1 group containing 6 sorted bindings.
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].groups.len(), 1);
        assert_eq!(columns[0].groups[0].1.len(), 6);

        // And bindings are sorted alphabetically by key display string.
        let keys: Vec<String> = columns[0].groups[0].1.iter().map(|(k, _)| k.display()).collect();
        assert_eq!(keys, vec!["a", "b", "c", "d", "e", "f"]);
    }

    #[test]
    fn flat_tall_splits_into_multiple_columns_on_overflow() {
        // Given 5 categories with 4 bindings each (20 total), frame height = 8 (available = 6).
        let groups = make_groups(&[
            ("A", &[("a", "a1"), ("b", "a2"), ("c", "a3"), ("d", "a4")]),
            ("B", &[("e", "b1"), ("f", "b2"), ("g", "b3"), ("h", "b4")]),
            ("C", &[("i", "c1"), ("j", "c2"), ("k", "c3"), ("l", "c4")]),
            ("D", &[("m", "d1"), ("n", "d2"), ("o", "d3"), ("p", "d4")]),
            ("E", &[("q", "e1"), ("r", "e2"), ("s", "e3"), ("t", "e4")]),
        ]);
        let config = make_config(DisplayMode::Flat, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 8, 80);

        // Then result has multiple columns (ceil(20/6) = 4).
        assert!(columns.len() >= 3);

        // And no bindings are dropped.
        assert_eq!(total_bindings(&columns), 20);

        // And alphabetical order flows left-to-right (first column has earliest letters).
        let first_key = columns[0].groups[0].1.first().unwrap().0.display();
        let last_key = columns
            .last()
            .unwrap()
            .groups[0]
            .1
            .last()
            .unwrap()
            .0
            .display();
        assert!(
            first_key < last_key,
            "first key '{first_key}' should sort before last key '{last_key}'"
        );
    }

    #[test]
    fn flat_wide_distributes_across_columns() {
        // Given 10 bindings, frame width = 100.
        let groups = make_groups(&[
            ("X", &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")]),
            ("Y", &[("f", "f"), ("g", "g"), ("h", "h"), ("i", "i"), ("j", "j")]),
        ]);
        let config = make_config(DisplayMode::Flat, LayoutStrategy::PreferWide);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 100);

        // Then result has 3 columns (3×30 + 2×6 = 102 > 96, fallback to col_width=28).
        assert_eq!(columns.len(), 3);

        // And bindings are distributed evenly: first column gets 4, rest get 3.
        assert_eq!(columns[0].groups[0].1.len(), 4);
        assert_eq!(columns[1].groups[0].1.len(), 3);
        assert_eq!(columns[2].groups[0].1.len(), 3);

        // And no bindings are dropped.
        assert_eq!(total_bindings(&columns), 10);
    }

    #[test]
    fn flat_mode_has_no_category_headers() {
        // Given groups with category names in Flat mode.
        let groups = make_groups(&[
            ("Nav", &[("h", "left"), ("l", "right")]),
            ("Edit", &[("d", "delete")]),
        ]);
        let config = make_config(DisplayMode::Flat, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 80);

        // Then every group in every column has an empty string category.
        for col in &columns {
            for (category, _) in &col.groups {
                assert!(
                    category.is_empty(),
                    "Expected empty category, got '{category}'"
                );
            }
        }
    }

    #[test]
    fn category_tall_no_bindings_dropped_with_many_bindings() {
        // Given 5 categories with 10 bindings each (50 bindings total, 55 rows), frame height = 12.
        let groups: Vec<BindingGroup<TestKey>> = (0..5)
            .map(|ci| {
                let cat = format!("Cat{ci}");
                BindingGroup {
                    category: cat,
                    bindings: (0..10)
                        .map(|i| Binding {
                            key: TestKey::Char(char::from(b'a' + ((i % 26) as u8))),
                            description: format!("action{i}"),
                        })
                        .collect(),
                }
            })
            .collect();
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 12, 200);

        // Then sum of all bindings across all columns == 50.
        assert_eq!(total_bindings(&columns), 50);
    }

    #[test]
    fn empty_groups_produces_no_bindings() {
        // Given an empty slice of groups.
        let groups: Vec<BindingGroup<TestKey>> = Vec::new();
        let config = make_config(DisplayMode::Category, LayoutStrategy::PreferTall);

        // When building columns.
        let columns = build_columns(&groups, &config, 20, 80);

        // Then no bindings are present (category_tall returns one empty column).
        assert_eq!(total_bindings(&columns), 0);
    }

    #[test]
    fn flatten_bindings_sorts_alphabetically() {
        // Given groups with keys 'c', 'a', 'b'.
        let groups = make_groups(&[
            ("X", &[("c", "third")]),
            ("Y", &[("a", "first")]),
            ("Z", &[("b", "second")]),
        ]);

        // When flattening bindings.
        let flat = flatten_bindings(&groups);

        // Then result is ordered by key display: a, b, c.
        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].0.display(), "a");
        assert_eq!(flat[1].0.display(), "b");
        assert_eq!(flat[2].0.display(), "c");
    }
}
