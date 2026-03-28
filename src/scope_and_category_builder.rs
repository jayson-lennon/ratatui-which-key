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

/// A builder for binding key sequences with a pre-configured scope and category.
///
/// This builder simplifies repetitive bindings by storing a scope and category
/// that are automatically applied to each [`bind`](Self::bind) call.
#[derive(Debug)]
pub struct ScopeAndCategoryBuilder<'a, K: Key, S, A, C> {
    keymap: &'a mut Keymap<K, S, A, C>,
    scope: S,
    category: C,
}

impl<'a, K: Key, S, A, C> ScopeAndCategoryBuilder<'a, K, S, A, C> {
    pub(super) fn new(keymap: &'a mut Keymap<K, S, A, C>, scope: S, category: C) -> Self {
        Self {
            keymap,
            scope,
            category,
        }
    }

    /// Binds a key sequence to an action using the configured scope and category.
    pub fn bind(&mut self, sequence: &str, action: A) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        self.keymap
            .bind(sequence, action, self.category.clone(), self.scope.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::KeyNode;
    use crate::test_utils::{TestAction, TestCategory, TestScope};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn bind_auto_applies_scope_and_category() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        let mut builder =
            ScopeAndCategoryBuilder::new(&mut keymap, TestScope::Global, TestCategory::Navigation);

        builder.bind("h", TestAction::Open);

        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())]);
        assert!(node.is_some());

        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Open);
            assert_eq!(entries[0].scope, TestScope::Global);
            assert_eq!(entries[0].category, TestCategory::Navigation);
        } else {
            panic!("Expected leaf node");
        }
    }

    #[test]
    fn chaining_works() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        let mut builder =
            ScopeAndCategoryBuilder::new(&mut keymap, TestScope::Insert, TestCategory::General);

        builder
            .bind("q", TestAction::Quit)
            .bind("s", TestAction::Save)
            .bind("o", TestAction::Open);

        let quit_node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())]);
        let save_node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty())]);
        let open_node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty())]);

        assert!(quit_node.is_some());
        assert!(save_node.is_some());
        assert!(open_node.is_some());

        if let Some(KeyNode::Leaf(entries)) = quit_node {
            assert_eq!(entries[0].action, TestAction::Quit);
            assert_eq!(entries[0].scope, TestScope::Insert);
            assert_eq!(entries[0].category, TestCategory::General);
        }

        if let Some(KeyNode::Leaf(entries)) = save_node {
            assert_eq!(entries[0].action, TestAction::Save);
            assert_eq!(entries[0].scope, TestScope::Insert);
            assert_eq!(entries[0].category, TestCategory::General);
        }

        if let Some(KeyNode::Leaf(entries)) = open_node {
            assert_eq!(entries[0].action, TestAction::Open);
            assert_eq!(entries[0].scope, TestScope::Insert);
            assert_eq!(entries[0].category, TestCategory::General);
        }
    }
}
