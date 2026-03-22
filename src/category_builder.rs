// Copyright (C) 2026 Jayson Lennon
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{Key, Keymap};

/// Builder for creating keybindings within a specific category.
///
/// Provides convenience methods that default the category, reducing
/// boilerplate when defining groups of related keybindings.
#[derive(Debug)]
pub struct CategoryBuilder<'a, K: Key, S, A, C> {
    keymap: &'a mut Keymap<K, S, A, C>,
    category: C,
}

impl<'a, K: Key, S, A, C> CategoryBuilder<'a, K, S, A, C> {
    pub(super) fn new(keymap: &'a mut Keymap<K, S, A, C>, category: C) -> Self {
        Self { keymap, category }
    }

    /// Adds a keybinding with the builder's category and an explicit scope.
    pub fn bind(&mut self, sequence: &str, action: A, scope: S) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        self.keymap
            .bind(sequence, action, self.category.clone(), scope);
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::test_utils::{TestAction, TestCategory, TestScope};
    use crate::{CrosstermKey, KeyNode};

    #[test]
    fn bind_with_navigation_category_works_correctly() {
        // Given a keymap with a category binding using bind() with Navigation category.
        let mut keymap = Keymap::new();
        let mut builder = CategoryBuilder::new(&mut keymap, TestCategory::Navigation);
        builder.bind("h", TestAction::Open, TestScope::Global);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);

        // Then the binding exists with Navigation category.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].category, TestCategory::Navigation);
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("Expected leaf node with Navigation category");
        }
    }

    #[test]
    fn chaining_binds_first_key() {
        // Given a keymap with a chained binding.
        let mut keymap = Keymap::new();
        let mut builder = CategoryBuilder::new(&mut keymap, TestCategory::General);
        builder.bind("q", TestAction::Quit, TestScope::Global);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);

        // Then the binding exists with the correct action.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Quit);
        } else {
            panic!("Expected leaf node for 'q'");
        }
    }

    #[test]
    fn chaining_binds_second_key() {
        // Given a keymap with a chained binding.
        let mut keymap = Keymap::new();
        let mut builder = CategoryBuilder::new(&mut keymap, TestCategory::Navigation);
        builder.bind("h", TestAction::Open, TestScope::Global);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);

        // Then the binding exists with the correct action.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("Expected leaf node for 'h'");
        }
    }

    #[test]
    fn chaining_binds_third_key() {
        // Given a keymap with a chained binding.
        let mut keymap = Keymap::new();
        let mut builder = CategoryBuilder::new(&mut keymap, TestCategory::General);
        builder.bind("s", TestAction::Save, TestScope::Global);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('s')]);

        // Then the binding exists with the correct action.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Save);
        } else {
            panic!("Expected leaf node for 's'");
        }
    }
}
