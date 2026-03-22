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

use crate::{Key, Keymap, parse_key_sequence};

/// A builder for binding key sequences under a shared prefix.
#[derive(Debug)]
pub struct GroupBuilder<'a, K: Key, S, A, C> {
    keymap: &'a mut Keymap<K, S, A, C>,
    prefix: Vec<K>,
}

impl<'a, K: Key, S, A, C> GroupBuilder<'a, K, S, A, C> {
    pub(super) fn new(keymap: &'a mut Keymap<K, S, A, C>, prefix: Vec<K>) -> Self {
        Self { keymap, prefix }
    }

    /// Binds a key sequence to an action, prepending the group's prefix.
    pub fn bind(&mut self, sequence: &str, action: A, category: C, scope: S) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        let keys = parse_key_sequence(sequence);
        if keys.is_empty() {
            return self;
        }
        let full_sequence: Vec<K> = self.prefix.iter().cloned().chain(keys).collect();
        self.keymap
            .insert_into_tree(&full_sequence, action, category, scope);
        self
    }

    /// Creates a nested group with a description and binds keys within it.
    ///
    /// The `prefix` is appended to the current prefix, and `bindings` receives
    /// a new `GroupBuilder` for the nested group.
    pub fn describe<F>(&mut self, prefix: &str, description: &'static str, bindings: F)
    where
        F: for<'b> FnOnce(&mut GroupBuilder<'b, K, S, A, C>),
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        let keys = parse_key_sequence(prefix);
        if keys.is_empty() {
            return;
        }
        let full_prefix: Vec<K> = self.prefix.iter().cloned().chain(keys).collect();
        self.keymap
            .ensure_branch_with_description(&full_prefix, description);
        let mut builder = GroupBuilder::new(self.keymap, full_prefix);
        bindings(&mut builder);
    }

    /// Adds a description to a prefix without creating nested bindings.
    pub fn describe_prefix(&mut self, prefix: &str, description: &'static str) -> &mut Self
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        let keys = parse_key_sequence(prefix);
        if keys.is_empty() {
            return self;
        }
        let full_prefix: Vec<K> = self.prefix.iter().cloned().chain(keys).collect();
        self.keymap
            .ensure_branch_with_description(&full_prefix, description);
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
    fn bind_combines_prefix_with_sequence() {
        // Given a keymap with a described prefix group.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("g", "goto");
        let mut builder = GroupBuilder::new(&mut keymap, vec![CrosstermKey::Char('g')]);

        // When binding under the prefix.
        builder.bind(
            "h",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the binding is at the combined path.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('h')]);
        assert!(node.is_some());

        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Quit);
            assert_eq!(entries[0].description, TestAction::Quit.to_string());
        } else {
            panic!("Expected leaf node with Quit action");
        }
    }

    #[test]
    fn describe_prefix_sets_nested_description() {
        // Given a keymap with a described prefix group.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("g", "goto");
        let mut builder = GroupBuilder::new(&mut keymap, vec![CrosstermKey::Char('g')]);

        // When adding a nested prefix description.
        builder.describe_prefix("c", "git commits");

        // Then the nested prefix has its description.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('c')]);
        assert!(node.is_some());

        if let Some(KeyNode::Branch { description, .. }) = node {
            assert_eq!(*description, "git commits");
        } else {
            panic!("Expected branch node with description");
        }
    }

    #[test]
    fn describe_creates_nested_group() {
        // Given a keymap with a described prefix group.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("g", "goto");
        let mut builder = GroupBuilder::new(&mut keymap, vec![CrosstermKey::Char('g')]);

        // When creating a nested group with bindings.
        builder.describe("c", "git commands", |nested| {
            nested.bind(
                "l",
                TestAction::Open,
                TestCategory::General,
                TestScope::Global,
            );
            nested.bind(
                "s",
                TestAction::Save,
                TestCategory::General,
                TestScope::Global,
            );
        });

        // Then the nested prefix has its description and bindings.
        let branch_node =
            keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('c')]);
        assert!(branch_node.is_some());

        if let Some(KeyNode::Branch { description, .. }) = branch_node {
            assert_eq!(*description, "git commands");
        } else {
            panic!("Expected branch node with description");
        }

        let leaf_l = keymap.get_node_at_path(&[
            CrosstermKey::Char('g'),
            CrosstermKey::Char('c'),
            CrosstermKey::Char('l'),
        ]);
        assert!(leaf_l.is_some());

        let leaf_s = keymap.get_node_at_path(&[
            CrosstermKey::Char('g'),
            CrosstermKey::Char('c'),
            CrosstermKey::Char('s'),
        ]);
        assert!(leaf_s.is_some());
    }
}
