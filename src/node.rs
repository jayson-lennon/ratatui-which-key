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

use crate::Key;
use std::borrow::Cow;

/// A single action entry in a leaf node.
#[derive(Debug, Clone)]
pub struct LeafEntry<S, A, C> {
    /// The action to execute.
    pub action: A,
    /// Human-readable description of the action.
    pub description: String,
    /// Category for grouping related actions.
    pub category: C,
    /// Scope where this action is valid.
    ///
    /// `None` means the binding is global — it applies in every scope.
    /// A specific-scope entry always wins over a global one for the same key.
    pub scope: Option<S>,
}

/// A node in the keybinding tree, either a leaf with actions or a branch with children.
#[derive(Debug, Clone)]
pub enum KeyNode<K: Key, S, A, C> {
    /// A terminal node containing one or more action entries.
    Leaf(Vec<LeafEntry<S, A, C>>),
    /// A non-terminal node with a description and child nodes.
    Branch {
        /// Description shown in the which-key popup.
        description: &'static str,
        /// Per-scope description overrides.
        scope_descriptions: Vec<(S, &'static str)>,
        /// Child key bindings.
        children: Vec<KeyChild<K, S, A, C>>,
        /// Leaf entries for scopes where this key is a terminal action.
        leaf_entries: Vec<LeafEntry<S, A, C>>,
        /// Explicit category override for this branch node.
        ///
        /// When set, [`KeyNode::category`] returns this value instead of
        /// recursing into children. Set via [`Keymap::describe_group_with_category`].
        category: Option<C>,
    },
}

impl<K: Key, S, A, C> KeyNode<K, S, A, C> {
    /// Returns the description of this node.
    pub fn description(&self, scope: &S) -> Cow<'_, str>
    where
        S: PartialEq,
    {
        match self {
            KeyNode::Leaf(entries) => entries
                .first()
                .map_or(Cow::Borrowed(""), |e| Cow::Borrowed(&e.description)),
            KeyNode::Branch {
                description,
                scope_descriptions,
                leaf_entries,
                ..
            } => {
                if let Some(entry) = leaf_entries
                    .iter()
                    .find(|e| e.scope.as_ref() == Some(scope))
                    .or_else(|| leaf_entries.iter().find(|e| e.scope.is_none()))
                {
                    Cow::Borrowed(&entry.description)
                } else if let Some((_, scoped_desc)) =
                    scope_descriptions.iter().find(|(s, _)| s == scope)
                {
                    Cow::Borrowed(scoped_desc)
                } else {
                    Cow::Borrowed(description)
                }
            }
        }
    }

    /// Returns `true` if this node is a branch with children.
    pub fn is_branch(&self) -> bool {
        matches!(self, KeyNode::Branch { .. })
    }

    /// Returns the category of this node.
    ///
    /// For leaves, returns the first entry's category.
    /// For branches with an explicit category, returns that.
    /// For branches without, returns `None`.
    #[cfg(test)]
    pub fn category(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            KeyNode::Leaf(entries) => entries.first().map(|e| e.category.clone()),
            KeyNode::Branch { category, .. } => category.clone(),
        }
    }
}


/// A key binding with its associated key and node.
#[derive(Debug, Clone)]
pub struct KeyChild<K: Key, S, A, C> {
    /// The key that triggers this binding.
    pub key: K,
    /// The node (leaf or branch) for this binding.
    pub node: KeyNode<K, S, A, C>,
}

impl<K: Key, S: Clone, A: Clone, C: Clone> KeyChild<K, S, A, C> {
    /// Creates a leaf key child with a single action entry.
    ///
    /// The `scope` is stored as `Some(scope)`; a scoped binding. Use
    /// [`Keymap::bind_global`](crate::Keymap::bind_global) to add a binding
    /// that applies in every scope.
    #[cfg(test)]
    pub fn leaf(key: K, action: A, description: String, category: C, scope: S) -> Self {
        let entry = LeafEntry {
            action,
            description,
            category,
            scope: Some(scope),
        };
        Self {
            key,
            node: KeyNode::Leaf(vec![entry]),
        }
    }

    /// Creates a branch key child with the given description and children.
    pub fn branch(key: K, description: &'static str, children: Vec<Self>) -> Self
    where
        C: Clone,
    {
        Self::branch_with_category(key, description, children, None)
    }

    /// Creates a branch key child with an explicit category.
    pub fn branch_with_category(
        key: K,
        description: &'static str,
        children: Vec<Self>,
        category: Option<C>,
    ) -> Self {
        Self {
            key,
            node: KeyNode::Branch {
                description,
                scope_descriptions: Vec::new(),
                children,
                leaf_entries: Vec::new(),
                category,
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{TestAction, TestCategory, TestKey, TestScope};

    fn create_test_leaf() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        KeyChild::leaf(
            TestKey::Char('q'),
            TestAction::Quit,
            "quit application".to_string(),
            TestCategory::General,
            TestScope::Normal,
        )
    }

    fn create_test_branch() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        let save_child = KeyChild::leaf(
            TestKey::Char('s'),
            TestAction::Save,
            "save file".to_string(),
            TestCategory::General,
            TestScope::Normal,
        );
        let open_child = KeyChild::leaf(
            TestKey::Char('o'),
            TestAction::Open,
            "open file".to_string(),
            TestCategory::Navigation,
            TestScope::Normal,
        );
        KeyChild::branch(
            TestKey::Char('f'),
            "file operations",
            vec![save_child, open_child],
        )
    }

    #[test]
    fn key_child_leaf_creates_leaf_node() {
        // Given a test leaf child.
        let child = create_test_leaf();

        // When checking if the node is a branch.
        let is_branch = child.node.is_branch();

        // Then it is not a branch (it's a leaf).
        assert!(!is_branch);
    }

    #[test]
    fn key_child_branch_creates_branch_node() {
        // Given a test branch child.
        let child = create_test_branch();

        // When checking if the node is a branch.
        let is_branch = child.node.is_branch();

        // Then it is a branch.
        assert!(is_branch);
    }

    #[test]
    fn key_node_category_returns_leaf_category() {
        // Given a test leaf child.
        let child = create_test_leaf();

        // When getting the node category.
        let category = child.node.category();

        // Then it returns the leaf category.
        assert_eq!(category, Some(TestCategory::General));
    }

    #[test]
    fn key_node_category_returns_none_for_branch() {
        // Given a test branch child.
        let child = create_test_branch();

        // When getting the node category.
        let category = child.node.category();

        // Then it returns none.
        assert_eq!(category, None);
    }

    #[test]
    fn branch_with_empty_children() {
        // Given a branch with empty children.
        let child: KeyChild<TestKey, TestScope, TestAction, TestCategory> =
            KeyChild::branch(TestKey::Char('x'), "empty branch", vec![]);

        // When checking the branch properties.
        let is_branch = child.node.is_branch();
        let description = child.node.description(&TestScope::Normal);

        // Then it is a branch with the correct description.
        assert!(is_branch);
        assert_eq!(description, "empty branch");
    }

    #[test]
    fn branch_description_returns_branch_desc_when_leaf_scope_differs() {
        // Given a branch with a leaf entry scoped to TestScope::Normal and children scoped to TestScope::Insert.
        let node: KeyNode<TestKey, TestScope, TestAction, TestCategory> = KeyNode::Branch {
            description: "source",
            scope_descriptions: vec![],
            children: vec![KeyChild::leaf(
                TestKey::Char('a'),
                TestAction::Save,
                "start analysis".to_string(),
                TestCategory::General,
                TestScope::Insert,
            )],
            leaf_entries: vec![LeafEntry {
                action: TestAction::Save,
                description: "start analysis".to_string(),
                category: TestCategory::General,
                scope: Some(TestScope::Normal),
            }],
            category: None,
        };

        // When querying the description with TestScope::Insert (which doesn't match the leaf entry scope).
        let result = node.description(&TestScope::Insert);

        // Then the branch's own description is returned, not the leaf entry's.
        assert_eq!(result, "source");
    }

    #[test]
    fn branch_description_returns_leaf_desc_when_leaf_scope_matches() {
        // Given a branch with a leaf entry scoped to TestScope::Normal.
        let node: KeyNode<TestKey, TestScope, TestAction, TestCategory> = KeyNode::Branch {
            description: "source",
            scope_descriptions: vec![],
            children: vec![KeyChild::leaf(
                TestKey::Char('a'),
                TestAction::Save,
                "start analysis".to_string(),
                TestCategory::General,
                TestScope::Insert,
            )],
            leaf_entries: vec![LeafEntry {
                action: TestAction::Save,
                description: "start analysis".to_string(),
                category: TestCategory::General,
                scope: Some(TestScope::Normal),
            }],
            category: None,
        };

        // When querying the description with TestScope::Normal (which matches the leaf entry scope).
        let result = node.description(&TestScope::Normal);

        // Then the leaf entry's description is returned.
        assert_eq!(result, "start analysis");
    }

    #[test]
    fn nested_branch_structure() {
        // Given a nested branch structure with an inner leaf, inner branch, and outer branch.
        let inner_leaf = KeyChild::leaf(
            TestKey::Char('d'),
            TestAction::Quit,
            "delete".to_string(),
            TestCategory::General,
            TestScope::Normal,
        );
        let inner_branch =
            KeyChild::branch(TestKey::Char('d'), "delete operations", vec![inner_leaf]);
        let outer_branch = KeyChild::branch(TestKey::Char('g'), "go to", vec![inner_branch]);

        // When checking the outer branch properties.
        let is_branch = outer_branch.node.is_branch();

        // Then the outer branch is a branch and contains the inner branch.
        assert!(is_branch);

        if let KeyNode::Branch { children, .. } = outer_branch.node {
            assert_eq!(children.len(), 1);
            assert!(children[0].node.is_branch());
        } else {
            panic!("expected branch node");
        }
    }
}
