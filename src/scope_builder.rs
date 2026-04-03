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

use crate::{Key, Keymap};

/// Builder for creating keybindings within a specific scope.
///
/// Provides convenience methods that default the scope, reducing
/// boilerplate when defining groups of related keybindings.
pub struct ScopeBuilder<'a, K: Key, S, A, C> {
    keymap: &'a mut Keymap<K, S, A, C>,
    scope: S,
}

impl<'a, K: Key, S, A, C> ScopeBuilder<'a, K, S, A, C> {
    pub(super) fn new(keymap: &'a mut Keymap<K, S, A, C>, scope: S) -> Self {
        Self { keymap, scope }
    }

    /// Adds a keybinding with explicit category.
    pub fn bind(&mut self, sequence: &str, action: A, category: C) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        self.keymap
            .bind(sequence, action, category, self.scope.clone());
        self
    }

    /// Sets a description for a prefix key group in this scope.
    ///
    /// The description is stored as a scope-specific override. When
    /// displaying bindings for this scope, this description takes
    /// priority over the default set by [`Keymap::describe_group`].
    pub fn describe_group(&mut self, prefix: &str, description: &'static str) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        self.keymap
            .describe_group_for_scope(prefix, description, self.scope.clone());
        self
    }

    /// Register a catch-all handler for this scope.
    ///
    /// The handler is invoked when a key doesn't match any binding.
    /// Returns `Some(action)` to dispatch an action, or `None` to dismiss.
    pub fn catch_all<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(K) -> Option<A> + Send + Sync + 'static,
        S: Clone + Ord,
        C: Clone,
    {
        self.keymap.register_catch_all(self.scope.clone(), handler);
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::test_utils::{TestAction, TestCategory, TestScope};
    use crate::KeyNode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn bind_with_navigation_category_works_correctly() {
        // Given a keymap with a scope binding using bind() with Navigation category.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("h", TestAction::Open, TestCategory::Navigation);

        // When looking up the binding.
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())]);

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
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("q", TestAction::Quit, TestCategory::General);

        // When looking up the binding.
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())]);

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
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("h", TestAction::Open, TestCategory::Navigation);

        // When looking up the binding.
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())]);

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
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("s", TestAction::Save, TestCategory::General);

        // When looking up the binding.
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty())]);

        // Then the binding exists with the correct action.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Save);
        } else {
            panic!("Expected leaf node for 's'");
        }
    }

    #[test]
    fn describe_group_sets_scope_specific_description() {
        // Given a scope builder for Global scope.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("ta", TestAction::Quit, TestCategory::General);
        builder.describe_group("t", "t-group");

        // When getting bindings for the Global scope.
        let bindings: Vec<crate::DisplayBinding<KeyEvent, TestCategory>> =
            keymap.get_bindings_for_scope(TestScope::Global);
        let t_binding = bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Global scope");

        // Then the scope-specific description is set.
        assert_eq!(t_binding.description, "t-group");
    }
}
