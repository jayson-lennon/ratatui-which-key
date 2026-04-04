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

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    parse_key_sequence, state::CatchAllHandler, Binding, BindingGroup, CategoryBuilder,
    DisplayBinding, GroupBuilder, Key, KeyChild, KeyNode, LeafEntry, NodeResult,
    ScopeAndCategoryBuilder, ScopeBuilder,
};

/// A hierarchical keymap that maps key sequences to actions with scope and category support.
///
/// The keymap stores bindings in a tree structure where multi-key sequences
/// create branch nodes and leaf nodes contain the actual actions.
///
/// # Type Parameters
///
/// * `K` - The key type (must implement [`Key`])
/// * `S` - The scope type for context-sensitive bindings
/// * `A` - The action type triggered by key sequences
/// * `C` - The category type for grouping bindings
pub struct Keymap<K: Key, S, A, C> {
    bindings: Vec<KeyChild<K, S, A, C>>,
    leader_key: K,
    catch_all_handlers: BTreeMap<S, CatchAllHandler<K, A>>,
    backend_handlers: Option<Box<dyn Any + Send + Sync>>,
}

impl<K: Key + Clone, S: Clone, A: Clone, C: Clone> Clone for Keymap<K, S, A, C> {
    fn clone(&self) -> Self {
        Self {
            bindings: self.bindings.clone(),
            leader_key: self.leader_key.clone(),
            catch_all_handlers: self.catch_all_handlers.clone(),
            backend_handlers: None,
        }
    }
}

impl<K: Key, S, A, C> std::fmt::Debug for Keymap<K, S, A, C>
where
    K: std::fmt::Debug,
    S: std::fmt::Debug,
    A: std::fmt::Debug,
    C: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keymap")
            .field("bindings", &self.bindings)
            .field("leader_key", &self.leader_key)
            .finish_non_exhaustive()
    }
}

impl<K: Key, S, A, C: Clone> Keymap<K, S, A, C> {
    /// Creates a new empty keymap with space as the leader key.
    #[must_use]
    pub fn new() -> Self
    where
        K: Clone,
        S: Clone,
        A: Clone,
    {
        let mut keymap = Self {
            bindings: Vec::new(),
            leader_key: K::space(),
            catch_all_handlers: BTreeMap::new(),
            backend_handlers: None,
        };
        keymap.describe_group("<leader>", "<leader>");
        keymap
    }

    /// Sets a custom leader key for this keymap.
    #[must_use]
    pub fn with_leader(mut self, leader_key: K) -> Self
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        let old_leader = self.leader_key.clone();
        self.leader_key = leader_key.clone();
        self.bindings.retain(|b| b.key != old_leader);
        self.describe_group("<leader>", "<leader>");
        self
    }

    /// Returns a reference to the leader key.
    #[must_use]
    pub fn leader_key(&self) -> &K {
        &self.leader_key
    }

    /// Returns a slice of all root-level key bindings.
    #[must_use]
    pub fn bindings(&self) -> &[KeyChild<K, S, A, C>] {
        &self.bindings
    }

    /// Binds a key sequence to an action with category and scope.
    pub fn bind(&mut self, sequence: &str, action: A, category: C, scope: S) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        let keys = parse_key_sequence(sequence, &self.leader_key);
        if keys.is_empty() {
            return self;
        }
        self.insert_into_tree(&keys, action, category, scope);
        self
    }

    pub(super) fn insert_into_tree(&mut self, keys: &[K], action: A, category: C, scope: S)
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        if keys.is_empty() {
            return;
        }
        let first_key = keys[0].clone();
        if let Some(child) = self.bindings.iter_mut().find(|c| c.key == first_key) {
            Self::insert_into_child(child, keys, action, category, scope);
        } else {
            let new_child = Self::build_tree(keys, action, category, scope);
            self.bindings.push(new_child);
        }
    }

    fn build_tree(keys: &[K], action: A, category: C, scope: S) -> KeyChild<K, S, A, C>
    where
        K: Clone,
        S: Clone,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        if keys.len() == 1 {
            let description = action.to_string();
            KeyChild::leaf(keys[0].clone(), action, description, category, scope)
        } else {
            let first = keys[0].clone();
            let rest = &keys[1..];
            let child = Self::build_tree(rest, action, category, scope);
            KeyChild::branch(first, "...", vec![child])
        }
    }

    fn insert_into_child(
        child: &mut KeyChild<K, S, A, C>,
        keys: &[K],
        action: A,
        category: C,
        scope: S,
    ) where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone + std::fmt::Display,
        C: Clone,
    {
        if keys.len() == 1 {
            let description = action.to_string();
            match &mut child.node {
                KeyNode::Leaf(entries) => {
                    if let Some(existing) = entries.iter_mut().find(|e| e.scope == scope) {
                        existing.action = action;
                        existing.description = description;
                        existing.category = category;
                    } else {
                        entries.push(LeafEntry {
                            action,
                            description,
                            category,
                            scope,
                        });
                    }
                }
                KeyNode::Branch { leaf_entries, .. } => {
                    if let Some(existing) = leaf_entries.iter_mut().find(|e| e.scope == scope) {
                        existing.action = action;
                        existing.description = description;
                        existing.category = category;
                    } else {
                        leaf_entries.push(LeafEntry {
                            action,
                            description,
                            category,
                            scope,
                        });
                    }
                }
            }
            return;
        }

        let next_key = keys[1].clone();
        match &mut child.node {
            KeyNode::Leaf(entries) => {
                let existing_entries = std::mem::take(entries);
                let new_child = Self::build_tree(&keys[1..], action, category, scope);
                child.node = KeyNode::Branch {
                    description: "...",
                    scope_descriptions: Vec::new(),
                    children: vec![new_child],
                    leaf_entries: existing_entries,
                };
            }
            KeyNode::Branch { children, .. } => {
                if let Some(next_child) = children.iter_mut().find(|c| c.key == next_key) {
                    Self::insert_into_child(next_child, &keys[1..], action, category, scope);
                } else {
                    let new_child = Self::build_tree(&keys[1..], action, category, scope);
                    children.push(new_child);
                }
            }
        }
    }

    /// Returns the node at the given key path, or `None` if not found.
    #[must_use]
    pub fn get_node_at_path(&self, keys: &[K]) -> Option<&KeyNode<K, S, A, C>>
    where
        K: PartialEq,
    {
        if keys.is_empty() {
            return None;
        }

        let child = self.bindings.iter().find(|c| c.key == keys[0])?;
        if keys.len() == 1 {
            return Some(&child.node);
        }
        Self::get_node_in_children(&child.node, &keys[1..])
    }

    fn get_node_in_children<'a>(
        node: &'a KeyNode<K, S, A, C>,
        keys: &[K],
    ) -> Option<&'a KeyNode<K, S, A, C>>
    where
        K: PartialEq,
    {
        match node {
            KeyNode::Leaf(_) => {
                if keys.is_empty() {
                    Some(node)
                } else {
                    None
                }
            }
            KeyNode::Branch { children, .. } => {
                if keys.is_empty() {
                    return Some(node);
                }
                let child = children.iter().find(|c| c.key == keys[0])?;
                if keys.len() == 1 {
                    Some(&child.node)
                } else {
                    Self::get_node_in_children(&child.node, &keys[1..])
                }
            }
        }
    }

    /// Returns the children at the given key path as (key, description) pairs.
    ///
    /// Returns `None` if the path leads to a leaf node or doesn't exist.
    /// Returns root bindings for an empty path.
    #[must_use]
    pub fn get_children_at_path(&self, keys: &[K], scope: &S) -> Option<Vec<(K, String)>>
    where
        K: PartialEq + Clone,
        S: PartialEq,
    {
        let node = if keys.is_empty() {
            return Some(
                self.bindings
                    .iter()
                    .map(|c| (c.key.clone(), c.node.description(scope)))
                    .collect(),
            );
        } else {
            self.get_node_at_path(keys)?
        };

        match node {
            KeyNode::Branch { children, .. } => Some(
                children
                    .iter()
                    .filter(|c| Self::has_bindings_for_scope(&c.node, scope))
                    .map(|c| (c.key.clone(), c.node.description(scope)))
                    .collect(),
            ),
            KeyNode::Leaf(_) => None,
        }
    }

    /// Returns `true` if the given key is a prefix key (leads to a branch node).
    pub fn is_prefix_key(&self, key: K) -> bool
    where
        K: PartialEq,
    {
        self.bindings
            .iter()
            .any(|c| c.key == key && c.node.is_branch())
    }

    /// Returns all bindings visible in the given scope as display bindings.
    pub fn get_bindings_for_scope(&self, scope: S) -> Vec<DisplayBinding<K, C>>
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        self.bindings
            .iter()
            .filter_map(|child| match &child.node {
                KeyNode::Leaf(entries) => {
                    entries
                        .iter()
                        .find(|entry| entry.scope == scope)
                        .map(|entry| DisplayBinding {
                            key: child.key.clone(),
                            description: entry.description.clone(),
                            category: entry.category.clone(),
                        })
                }
                KeyNode::Branch { leaf_entries, .. } => {
                    if let Some(entry) = leaf_entries.iter().find(|e| e.scope == scope) {
                        Some(DisplayBinding {
                            key: child.key.clone(),
                            description: entry.description.clone(),
                            category: entry.category.clone(),
                        })
                    } else {
                        Self::find_category_in_children(&child.node, &scope).map(|category| {
                            DisplayBinding {
                                key: child.key.clone(),
                                description: child.node.description(&scope),
                                category,
                            }
                        })
                    }
                }
            })
            .collect()
    }

    fn find_category_in_children(node: &KeyNode<K, S, A, C>, scope: &S) -> Option<C>
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        match node {
            KeyNode::Leaf(entries) => entries
                .iter()
                .find(|e| &e.scope == scope)
                .map(|e| e.category.clone()),
            KeyNode::Branch { children, .. } => children
                .iter()
                .find_map(|c| Self::find_category_in_children_recursive(&c.node, scope)),
        }
    }

    fn find_category_in_children_recursive(node: &KeyNode<K, S, A, C>, scope: &S) -> Option<C>
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        match node {
            KeyNode::Leaf(entries) => entries
                .iter()
                .find(|e| &e.scope == scope)
                .map(|e| e.category.clone()),
            KeyNode::Branch { children, .. } => children
                .iter()
                .find_map(|c| Self::find_category_in_children_recursive(&c.node, scope)),
        }
    }

    /// Returns bindings grouped by category for the given scope.
    #[must_use]
    pub fn bindings_for_scope(&self, scope: S) -> Vec<BindingGroup<K>>
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone + std::fmt::Display,
    {
        let bindings = self.get_bindings_for_scope(scope);
        let groups: std::collections::BTreeMap<String, Vec<Binding<K>>> = bindings
            .iter()
            .map(|b| {
                (
                    format!("{}", b.category),
                    Binding {
                        key: b.key.clone(),
                        description: b.description.clone(),
                    },
                )
            })
            .fold(std::collections::BTreeMap::new(), |mut acc, (cat, bind)| {
                acc.entry(cat).or_default().push(bind);
                acc
            });
        groups
            .into_iter()
            .map(|(category, bindings)| BindingGroup { category, bindings })
            .collect()
    }

    /// Returns child bindings at the given key path.
    ///
    /// Returns `None` if the path leads to a leaf node or doesn't exist.
    #[must_use]
    pub fn children_at_path(&self, keys: &[K], scope: &S) -> Option<Vec<Binding<K>>>
    where
        K: Clone + PartialEq,
        S: PartialEq,
    {
        self.get_children_at_path(keys, scope).map(|children| {
            children
                .into_iter()
                .map(|(key, description)| Binding { key, description })
                .collect()
        })
    }

    fn has_bindings_for_scope(node: &KeyNode<K, S, A, C>, scope: &S) -> bool
    where
        S: PartialEq,
        C: Clone,
    {
        match node {
            KeyNode::Leaf(entries) => entries.iter().any(|e| &e.scope == scope),
            KeyNode::Branch {
                children,
                leaf_entries,
                ..
            } => {
                leaf_entries.iter().any(|e| &e.scope == scope)
                    || children
                        .iter()
                        .any(|c| Self::has_bindings_for_scope(&c.node, scope))
            }
        }
    }

    /// Navigates to the given key path and returns the result.
    ///
    /// Returns `None` if the path doesn't exist.
    ///
    /// Returns [`NodeResult::Branch`] with children for branch nodes.
    ///
    /// Returns [`NodeResult::Leaf`] with the action for leaf nodes.
    #[must_use]
    pub fn navigate(&self, keys: &[K], scope: &S) -> Option<NodeResult<K, A>>
    where
        K: Clone + PartialEq,
        A: Clone,
        S: PartialEq,
        C: Clone,
    {
        let node = self.get_node_at_path(keys)?;
        match node {
            KeyNode::Branch {
                children,
                leaf_entries,
                ..
            } => {
                if let Some(entry) = leaf_entries.iter().find(|e| &e.scope == scope) {
                    Some(NodeResult::Leaf {
                        action: entry.action.clone(),
                    })
                } else if Self::has_bindings_for_scope(node, scope) {
                    Some(NodeResult::Branch {
                        children: children
                            .iter()
                            .filter(|c| Self::has_bindings_for_scope(&c.node, scope))
                            .map(|c| Binding {
                                key: c.key.clone(),
                                description: c.node.description(scope),
                            })
                            .collect(),
                    })
                } else {
                    None
                }
            }
            KeyNode::Leaf(entries) => {
                entries
                    .iter()
                    .find(|e| &e.scope == scope)
                    .map(|e| NodeResult::Leaf {
                        action: e.action.clone(),
                    })
            }
        }
    }

    /// Sets a description for a prefix key group.
    ///
    /// Creates a branch node at the prefix path if it doesn't exist,
    /// or updates the description of an existing placeholder ("...").
    pub fn describe_group(&mut self, prefix: &str, description: &'static str) -> &mut Self
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        let keys = parse_key_sequence(prefix, &self.leader_key);
        if keys.is_empty() {
            return self;
        }
        self.ensure_branch_with_description(&keys, description);
        self
    }

    /// Sets a scope-specific description for a prefix key group.
    ///
    /// Creates a branch node at the prefix path if it doesn't exist,
    /// then adds a per-scope description override. When displaying
    /// bindings for the given scope, this description takes priority
    /// over the default set by [`describe_group`](Self::describe_group).
    pub fn describe_group_for_scope(
        &mut self,
        prefix: &str,
        description: &'static str,
        scope: S,
    ) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        let keys = parse_key_sequence(prefix, &self.leader_key);
        if keys.is_empty() {
            return self;
        }
        self.ensure_branch_with_scope_description(&keys, description, scope);
        self
    }

    pub(super) fn ensure_branch_with_description(&mut self, keys: &[K], description: &'static str)
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        if keys.is_empty() {
            return;
        }
        let first_key = keys[0].clone();
        if let Some(child) = self.bindings.iter_mut().find(|c| c.key == first_key) {
            Self::ensure_branch_in_child(child, keys, description);
        } else {
            let new_child = Self::build_branch_tree(keys, description);
            self.bindings.push(new_child);
        }
    }

    fn build_branch_tree(keys: &[K], description: &'static str) -> KeyChild<K, S, A, C>
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        if keys.len() == 1 {
            KeyChild::branch(keys[0].clone(), description, Vec::new())
        } else {
            let first = keys[0].clone();
            let rest = &keys[1..];
            let child = Self::build_branch_tree(rest, description);
            KeyChild::branch(first, description, vec![child])
        }
    }

    fn ensure_final_key_is_branch(child: &mut KeyChild<K, S, A, C>, description: &'static str) {
        match &mut child.node {
            KeyNode::Branch {
                description: desc, ..
            } => {
                if *desc == "..." {
                    *desc = description;
                }
            }
            KeyNode::Leaf(_) => {
                child.node = KeyNode::Branch {
                    description,
                    scope_descriptions: Vec::new(),
                    children: Vec::new(),
                    leaf_entries: Vec::new(),
                };
            }
        }
    }

    fn ensure_branch_in_child(
        child: &mut KeyChild<K, S, A, C>,
        keys: &[K],
        description: &'static str,
    ) where
        K: Clone,
        S: Clone,
        A: Clone,
    {
        if keys.len() == 1 {
            Self::ensure_final_key_is_branch(child, description);
            return;
        }

        let next_key = keys[1].clone();
        let remaining = &keys[1..];

        match &mut child.node {
            KeyNode::Leaf(_) => {
                let new_child = Self::build_branch_tree(remaining, description);
                child.node = KeyNode::Branch {
                    description,
                    scope_descriptions: Vec::new(),
                    children: vec![new_child],
                    leaf_entries: Vec::new(),
                };
            }
            KeyNode::Branch {
                description: desc,
                children,
                ..
            } => {
                if *desc == "..." {
                    *desc = description;
                }

                if let Some(next_child) = children.iter_mut().find(|c| c.key == next_key) {
                    Self::ensure_branch_in_child(next_child, remaining, description);
                } else {
                    let new_child = Self::build_branch_tree(remaining, description);
                    children.push(new_child);
                }
            }
        }
    }

    fn ensure_branch_with_scope_description(
        &mut self,
        keys: &[K],
        description: &'static str,
        scope: S,
    ) where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        if keys.is_empty() {
            return;
        }
        let first_key = keys[0].clone();
        if let Some(child) = self.bindings.iter_mut().find(|c| c.key == first_key) {
            Self::set_scope_description_in_child(child, keys, description, scope);
        } else {
            let mut new_child = Self::build_branch_tree(keys, "...");
            if let KeyNode::Branch {
                scope_descriptions, ..
            } = &mut new_child.node
            {
                scope_descriptions.push((scope, description));
            }
            self.bindings.push(new_child);
        }
    }

    fn set_scope_description_in_child(
        child: &mut KeyChild<K, S, A, C>,
        keys: &[K],
        description: &'static str,
        scope: S,
    ) where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
    {
        if keys.len() == 1 {
            Self::set_scope_description_on_final_key(child, description, scope);
            return;
        }

        let remaining = &keys[1..];

        match &mut child.node {
            KeyNode::Leaf(_) => {
                let mut new_child = Self::build_branch_tree(remaining, "...");
                if let KeyNode::Branch {
                    scope_descriptions, ..
                } = &mut new_child.node
                {
                    scope_descriptions.push((scope, description));
                }
                child.node = KeyNode::Branch {
                    description: "...",
                    scope_descriptions: Vec::new(),
                    children: vec![new_child],
                    leaf_entries: Vec::new(),
                };
            }
            KeyNode::Branch { children, .. } => {
                let next_key = keys[1].clone();
                if let Some(next_child) = children.iter_mut().find(|c| c.key == next_key) {
                    Self::set_scope_description_in_child(next_child, remaining, description, scope);
                } else {
                    let mut new_child = Self::build_branch_tree(remaining, "...");
                    if let KeyNode::Branch {
                        scope_descriptions, ..
                    } = &mut new_child.node
                    {
                        scope_descriptions.push((scope, description));
                    }
                    children.push(new_child);
                }
            }
        }
    }

    fn set_scope_description_on_final_key(
        child: &mut KeyChild<K, S, A, C>,
        description: &'static str,
        scope: S,
    ) where
        S: PartialEq,
    {
        match &mut child.node {
            KeyNode::Branch {
                scope_descriptions, ..
            } => {
                if let Some(entry) = scope_descriptions.iter_mut().find(|(s, _)| *s == scope) {
                    *entry = (scope, description);
                } else {
                    scope_descriptions.push((scope, description));
                }
            }
            KeyNode::Leaf(_) => {
                child.node = KeyNode::Branch {
                    description: "...",
                    scope_descriptions: vec![(scope, description)],
                    children: Vec::new(),
                    leaf_entries: Vec::new(),
                };
            }
        }
    }

    /// Creates a group of bindings under a prefix with a description.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_which_key::Keymap;
    /// use crossterm::event::KeyEvent;
    /// # // Define your action type
    /// # #[derive(Debug, Clone)]
    /// # enum Action { Quit, Save }
    ///
    /// # impl std::fmt::Display for Action {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    /// #         match self {
    /// #             Action::Quit => write!(f, "quit"),
    /// #             Action::Save => write!(f, "save"),
    /// #         }
    /// #     }
    /// # }
    ///
    /// # // Define your scope type
    /// # #[derive(Debug, Clone, PartialEq)]
    /// # enum Scope { Global, Insert }
    ///
    /// # // Define keybind categories
    /// # #[derive(Debug, Clone, PartialEq)]
    /// # enum Category { General, Navigation }
    ///
    /// # let mut keymap: Keymap<KeyEvent, Scope, Action, Category> = Keymap::new();
    /// keymap.group("g", "general", |b: &mut ratatui_which_key::GroupBuilder<KeyEvent, Scope, Action, Category>| {
    ///     // keybind is `gq`
    ///     b.bind("q", Action::Quit, Category::General, Scope::Global)
    ///     // keybind is `gs`
    ///      .bind("s", Action::Save, Category::General, Scope::Global);
    /// });
    /// ```
    pub fn group<F>(&mut self, prefix: &str, description: &'static str, bindings: F) -> &mut Self
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
        F: FnOnce(&mut GroupBuilder<K, S, A, C>),
    {
        let prefix_keys = parse_key_sequence(prefix, &self.leader_key);
        if prefix_keys.is_empty() {
            return self;
        }
        self.ensure_branch_with_description(&prefix_keys, description);
        let mut builder = GroupBuilder::new(self, prefix_keys);
        bindings(&mut builder);
        self
    }

    /// Creates a scope builder for adding bindings within a specific scope.
    pub fn scope<F>(&mut self, scope: S, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut ScopeBuilder<K, S, A, C>),
    {
        let mut builder = ScopeBuilder::new(self, scope);
        bindings(&mut builder);
        self
    }

    /// Creates a category builder for adding bindings within a specific category.
    pub fn category<F>(&mut self, category: C, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut CategoryBuilder<'_, K, S, A, C>),
    {
        let mut builder = CategoryBuilder::new(self, category);
        bindings(&mut builder);
        self
    }

    /// Creates a combined scope and category builder for adding bindings.
    pub fn scope_and_category<F>(&mut self, scope: S, category: C, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut ScopeAndCategoryBuilder<'_, K, S, A, C>),
    {
        let mut builder = ScopeAndCategoryBuilder::new(self, scope, category);
        bindings(&mut builder);
        self
    }

    /// Register a catch-all handler for a scope.
    ///
    /// The handler is invoked when a key doesn't match any binding in the given scope.
    /// Returns `Some(action)` to dispatch an action, or `None` to dismiss.
    pub fn register_catch_all<F>(&mut self, scope: S, handler: F)
    where
        S: Ord,
        F: Fn(K) -> Option<A> + Send + Sync + 'static,
    {
        self.catch_all_handlers.insert(scope, Arc::new(handler));
    }

    /// Get all catch-all handlers.
    pub fn catch_all_handlers(&self) -> &BTreeMap<S, CatchAllHandler<K, A>> {
        &self.catch_all_handlers
    }

    /// Set backend-specific handlers (used by backend extensions).
    pub fn set_backend_handlers<H: Any + Send + Sync>(&mut self, handlers: H) {
        self.backend_handlers = Some(Box::new(handlers));
    }

    /// Get backend-specific handlers (used by backend extensions).
    pub fn backend_handlers<H: Any + Send + Sync>(&self) -> Option<&H> {
        self.backend_handlers.as_ref()?.downcast_ref::<H>()
    }
}

impl<K: Key, S: Clone, A: Clone, C: Clone> Default for Keymap<K, S, A, C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::keymap_with_binding;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[derive(Debug, Clone, PartialEq)]
    enum TestAction {
        Quit,
        Save,
        Open,
    }

    impl std::fmt::Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::Quit => write!(f, "quit"),
                TestAction::Save => write!(f, "save"),
                TestAction::Open => write!(f, "open"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestScope {
        Global,
        Insert,
        Normal,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestCategory {
        General,
        Navigation,
    }

    fn get_leaf_entries<S: Clone + PartialEq, A: Clone, C: Clone + PartialEq>(
        keymap: &Keymap<KeyEvent, S, A, C>,
        key: KeyEvent,
    ) -> Vec<(S, A, C, String)> {
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == key)
            .expect("binding");
        if let KeyNode::Leaf(entries) = &child.node {
            entries
                .iter()
                .map(|e| {
                    (
                        e.scope.clone(),
                        e.action.clone(),
                        e.category.clone(),
                        e.description.clone(),
                    )
                })
                .collect()
        } else {
            panic!("expected leaf node");
        }
    }

    fn get_leaf_entry_count<S: Clone + PartialEq, A: Clone, C: Clone + PartialEq>(
        keymap: &Keymap<KeyEvent, S, A, C>,
        key: KeyEvent,
    ) -> usize {
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == key)
            .expect("binding");
        if let KeyNode::Leaf(entries) = &child.node {
            entries.len()
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn new_creates_empty_keymap_with_space_leader() {
        // Given no key bindings.
        // (Keymap::new() creates an empty keymap)

        // When creating a new keymap.
        let keymap: Keymap<KeyEvent, (), TestAction, TestCategory> = Keymap::new();

        // Then it has a space leader key and leader group binding.
        assert_eq!(
            keymap.leader_key(),
            &KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty())
        );
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn with_leader_creates_keymap_with_custom_leader() {
        // Given no key bindings.

        // When creating a keymap with a custom leader key.
        let keymap: Keymap<KeyEvent, (), TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));

        // Then it uses the custom leader and has leader group binding.
        assert_eq!(
            keymap.leader_key(),
            &KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())
        );
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn bind_with_leader_resolves_to_custom_leader_key() {
        // Given a keymap with a custom leader key.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        // When binding <leader>gg.
        keymap.bind(
            "<leader>gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the binding exists at path ['a', 'g', 'g'] (not space).
        assert_eq!(keymap.bindings().len(), 1);
        let first = &keymap.bindings()[0];
        assert_eq!(
            first.key,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );
        if let KeyNode::Branch { children, .. } = &first.node {
            assert_eq!(children.len(), 1);
            let second = &children[0];
            assert_eq!(
                second.key,
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
            );
            if let KeyNode::Branch { children, .. } = &second.node {
                assert_eq!(children.len(), 1);
                let third = &children[0];
                assert_eq!(
                    third.key,
                    KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
                );
                assert!(matches!(third.node, KeyNode::Leaf(_)));
            } else {
                panic!("expected branch node at second level");
            }
        } else {
            panic!("expected branch node at first level");
        }
    }

    #[test]
    fn navigate_with_custom_leader_key() {
        // Given a keymap with a custom leader key 'a'.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        // And binding <leader>gg to Quit.
        keymap.bind(
            "<leader>gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When navigating to ['a'].
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Global,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        // Then it returns a Leaf with the Quit action.
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Quit
            })
        ));
    }

    #[test]
    fn scope_and_category_bind_with_custom_leader() {
        // Given a keymap with a custom leader key 'a'.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        // And binding <leader>gg to Quit using scope_and_category builder.
        keymap.scope_and_category(TestScope::Global, TestCategory::Navigation, |g| {
            g.bind("<leader>gg", TestAction::Quit);
        });

        // When navigating to ['a'].
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Global,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        // Then it returns a Leaf with the Quit action.
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Quit
            })
        ));
    }

    #[test]
    fn demo_setup_with_custom_leader_and_zxc() {
        // Given a keymap with a custom leader key 'a'.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        // And binding both <leader>gg to Quit and zxc to Save.
        keymap.scope_and_category(TestScope::Global, TestCategory::Navigation, |g| {
            g.bind("<leader>gg", TestAction::Quit)
                .bind("zxc", TestAction::Save);
        });

        // When checking the root-level bindings count.
        // Then there are exactly two root-level bindings ('a' and 'z').
        assert_eq!(keymap.bindings().len(), 2);

        // When navigating to ['z'].
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty())],
            &TestScope::Global,
        );
        // Then it returns a Branch (zxc works).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a'].
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Global,
        );
        // Then it returns a Branch (agg should work too).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        // Then it returns a Leaf with the Quit action.
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Quit
            })
        ));
    }

    #[test]
    fn navigate_with_normal_scope_and_custom_leader() {
        // Given a keymap with a custom leader key 'a'.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));

        // And binding <leader>gg to Quit using Normal scope.
        keymap.scope_and_category(TestScope::Normal, TestCategory::Navigation, |g| {
            g.bind("<leader>gg", TestAction::Quit);
        });

        // When navigating to ['a'].
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Normal,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Normal,
        );
        // Then it returns a Branch (not None).
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // When navigating to ['a', 'g', 'g'].
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()),
            ],
            &TestScope::Normal,
        );
        // Then it returns a Leaf with the Quit action.
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Quit
            })
        ));
    }

    #[test]
    fn bind_single_key_creates_leaf_node() {
        // Given a keymap with a single binding.
        let keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // Then a leaf node is created with the binding (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
        let child = &keymap.bindings()[1];
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())
        );
        assert!(matches!(child.node, KeyNode::Leaf(_)));
    }

    #[test]
    fn multi_key_binding_count_is_one() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then there is exactly one binding (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn multi_key_root_key_is_g() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root key is 'g' (find it, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
        );
    }

    #[test]
    fn multi_key_root_is_branch() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root node is a branch (find 'g', not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert!(child.node.is_branch());
    }

    #[test]
    fn multi_key_second_level_count_is_one() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch has one child (find 'g', not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn multi_key_second_level_key_is_g() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the second level key is 'g' (find 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(
                children[0].key,
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
            );
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn multi_key_second_level_is_leaf() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the second level node is a leaf (find 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            assert!(matches!(children[0].node, KeyNode::Leaf(_)));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn same_scope_multi_entry_count_is_two() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );

        let count =
            get_leaf_entry_count(&keymap, KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));

        assert_eq!(count, 2);
    }

    #[test]
    fn same_scope_multi_entry_first_scope_is_global() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );

        let entries = get_leaf_entries(&keymap, KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));

        assert_eq!(entries[0].0, TestScope::Global);
    }

    #[test]
    fn same_scope_multi_entry_second_scope_is_insert() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );

        let entries = get_leaf_entries(&keymap, KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));

        assert_eq!(entries[1].0, TestScope::Insert);
    }

    #[test]
    fn same_scope_update_entry_count_is_one() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            TestCategory::Navigation,
            TestScope::Global,
        );

        let count = get_leaf_entry_count(
            &keymap,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
        );

        assert_eq!(count, 1);
    }

    #[test]
    fn same_scope_update_action_is_save() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(
            &keymap,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
        );

        assert_eq!(entries[0].1, TestAction::Save);
    }

    #[test]
    fn same_scope_update_description_is_save() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(
            &keymap,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
        );

        assert_eq!(entries[0].3, "save");
    }

    #[test]
    fn same_scope_update_category_is_navigation() {
        let mut keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(
            &keymap,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()),
        );

        assert_eq!(entries[0].2, TestCategory::Navigation);
    }

    #[test]
    fn bind_empty_sequence_does_nothing() {
        // Given a keymap (which starts with a leader description).
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        let initial_count = keymap.bindings().len();

        // When binding an empty sequence.
        keymap.bind(
            "",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // Then nothing additional is bound.
        assert_eq!(keymap.bindings().len(), initial_count);
    }

    #[test]
    fn branch_extension_binding_count_is_one() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then there is still only one binding at the root (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn branch_extension_root_key_is_g() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root key is 'g' (find it, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
        );
    }

    #[test]
    fn branch_extension_children_count_is_two() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch has two children (find the 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 2);
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn branch_extension_includes_d_key() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch includes the 'd' key (find the 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            let keys: Vec<_> = children.iter().map(|c| c.key).collect();
            assert!(keys.contains(&KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty())));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn branch_extension_preserves_g_key() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch still includes the 'g' key (find the 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            let keys: Vec<_> = children.iter().map(|c| c.key).collect();
            assert!(keys.contains(&KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_converts_leaf_to_branch_when_extending() {
        // Given a keymap with a leaf node.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "g",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding a multi-key sequence starting with that key.
        keymap.bind(
            "gg",
            TestAction::Open,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the leaf is converted to a branch (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert!(child.node.is_branch());

        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 1);
            assert_eq!(
                children[0].key,
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
            );
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_returns_self_for_chaining() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When chaining multiple bind calls.
        keymap
            .bind(
                "q",
                TestAction::Quit,
                TestCategory::General,
                TestScope::Global,
            )
            .bind(
                "w",
                TestAction::Save,
                TestCategory::General,
                TestScope::Global,
            );

        // Then both bindings are added (plus leader group).
        assert_eq!(keymap.bindings().len(), 3);
    }

    #[test]
    fn get_node_at_path_returns_none_for_empty_keys() {
        // Given an empty keymap.
        let keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When getting a node with an empty path.
        let result = keymap.get_node_at_path(&[]);

        // Then no node is returned.
        assert!(result.is_none());
    }

    #[test]
    fn get_node_at_path_returns_none_for_nonexistent_key() {
        // Given an empty keymap.
        let keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When getting a node with a nonexistent key.
        let result =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())]);

        // Then no node is returned.
        assert!(result.is_none());
    }

    #[test]
    fn get_children_at_path_returns_root_bindings_for_empty_keys() {
        // Given a keymap with multiple bindings.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "w",
            TestAction::Save,
            TestCategory::General,
            TestScope::Global,
        );

        // When getting children at an empty path.
        let result = keymap.get_children_at_path(&[], &TestScope::Global);

        // Then root bindings are returned (plus leader group).
        assert!(result.is_some());
        let children = result.unwrap();
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn get_children_at_path_returns_none_for_leaf() {
        // Given a keymap with a leaf node.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // When getting children at the leaf path.
        let result = keymap.get_children_at_path(
            &[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())],
            &TestScope::Global,
        );

        // Then none is returned.
        assert!(result.is_none());
    }

    #[test]
    fn is_prefix_key_returns_false_for_leaf() {
        // Given a keymap with a leaf node.
        let keymap = keymap_with_binding::<KeyEvent, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // When checking if 'q' is a prefix key.
        let result = keymap.is_prefix_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));

        // Then it returns false.
        assert!(!result);
    }

    #[test]
    fn get_bindings_for_scope_filters_by_exact_scope() {
        // Given a keymap with bindings in different scopes.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "w",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );
        keymap.bind(
            "e",
            TestAction::Open,
            TestCategory::General,
            TestScope::Normal,
        );

        // When getting bindings for a specific scope.
        let result = keymap.get_bindings_for_scope(TestScope::Insert);

        // Then only bindings with that exact scope are returned.
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].key,
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())
        );
        assert_eq!(result[0].description, "save");
    }

    #[test]
    fn get_bindings_for_scope_returns_empty_for_no_matches() {
        // Given a keymap with bindings in Global scope.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // When getting bindings for a different scope.
        let result = keymap.get_bindings_for_scope(TestScope::Insert);

        // Then an empty list is returned.
        assert!(result.is_empty());
    }

    #[test]
    fn get_bindings_for_scope_includes_branches() {
        // Given a keymap with both branches and leaves.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.bind(
            "q",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // When getting bindings for Global scope.
        let result = keymap.get_bindings_for_scope(TestScope::Global);

        // Then both branch and leaf bindings are included.
        assert_eq!(result.len(), 2);
        let keys: Vec<_> = result.iter().map(|b| b.key).collect();
        assert!(keys.contains(&KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())));
        assert!(keys.contains(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())));
    }

    #[test]
    fn describe_prefix_creates_branch_at_path() {
        // Given an empty keymap.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        // When describing a prefix.
        keymap.describe_group("g", "go commands");

        // Then a branch is created with the description (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
        );
        assert!(child.node.is_branch());

        if let KeyNode::Branch {
            description,
            children,
            ..
        } = &child.node
        {
            assert_eq!(*description, "go commands");
            assert!(children.is_empty());
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn describe_prefix_updates_existing_placeholder_description() {
        // Given a keymap with an existing binding.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When describing the prefix.
        keymap.describe_group("g", "go commands");

        // Then the description is updated (find the 'g' binding, not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        if let KeyNode::Branch { description, .. } = &child.node {
            assert_eq!(*description, "go commands");
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn single_level_branch_count_is_one() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn single_level_branch_key_is_a() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );
    }

    #[test]
    fn single_level_branch_description_is_set() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        if let KeyNode::Branch { description, .. } = &child.node {
            assert_eq!(*description, "single command");
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn single_level_branch_has_no_children() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        if let KeyNode::Branch { children, .. } = &keymap.bindings()[0].node {
            assert!(children.is_empty());
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn nested_branches_first_key_is_a() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())
        );
    }

    #[test]
    fn nested_branches_first_description_is_nested_command() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        if let KeyNode::Branch { description, .. } = &child.node {
            assert_eq!(*description, "nested command");
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_second_key_is_b() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(
                children[0].key,
                KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty())
            );
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_third_key_is_c() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            if let KeyNode::Branch { children, .. } = &children[0].node {
                assert_eq!(
                    children[0].key,
                    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty())
                );
            } else {
                panic!("expected branch node at 'b'");
            }
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_third_level_is_leaf() {
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()))
            .expect("a binding");
        if let KeyNode::Branch { children, .. } = &child.node {
            if let KeyNode::Branch { children, .. } = &children[0].node {
                if let KeyNode::Branch { children, .. } = &children[0].node {
                    assert!(children.is_empty());
                } else {
                    panic!("expected branch node at 'c'");
                }
            } else {
                panic!("expected branch node at 'b'");
            }
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn describe_prefix_and_bind_work_together() {
        // Given an empty keymap.

        // When using describe and bind together.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("g", "go commands").bind(
            "gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the prefix is described and the binding is under it (find 'g', not leader).
        let child = keymap
            .bindings()
            .iter()
            .find(|c| c.key == KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()))
            .expect("g binding");
        assert_eq!(
            child.key,
            KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
        );

        if let KeyNode::Branch {
            description,
            children,
            ..
        } = &child.node
        {
            assert_eq!(*description, "go commands");
            assert_eq!(children.len(), 1);
            assert_eq!(
                children[0].key,
                KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())
            );
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn describe_prefix_empty_string_does_nothing() {
        // Given an empty keymap.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();

        // When describing an empty prefix.
        keymap.describe_group("", "empty");

        // Then no additional bindings are created (only leader group exists).
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn scope_groups_binds_first_key_count_is_one() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, TestCategory::General);
        });

        // When checking bindings count.
        // Then there is exactly one binding (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn scope_groups_binds_first_key_node_exists() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, TestCategory::General);
        });

        // When looking up the node at path ['q'].
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_first_key_entry_count_is_one() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, TestCategory::General);
        });

        // When checking entry count at path ['q'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then there is exactly one entry.
            assert_eq!(entries.len(), 1);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_first_key_scope_is_global() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, TestCategory::General);
        });

        // When checking the scope at path ['q'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the scope is Global.
            assert_eq!(entries[0].scope, TestScope::Global);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_first_key_action_is_quit() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, TestCategory::General);
        });

        // When checking the action at path ['q'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the action is Quit.
            assert_eq!(entries[0].action, TestAction::Quit);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_second_key_count_is_one() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, TestCategory::General);
        });

        // When checking bindings count.
        // Then there is exactly one binding (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn scope_groups_binds_second_key_node_exists() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, TestCategory::General);
        });

        // When looking up the node at path ['w'].
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_second_key_entry_count_is_one() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, TestCategory::General);
        });

        // When checking entry count at path ['w'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then there is exactly one entry.
            assert_eq!(entries.len(), 1);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_second_key_scope_is_global() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, TestCategory::General);
        });

        // When checking the scope at path ['w'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the scope is Global.
            assert_eq!(entries[0].scope, TestScope::Global);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_second_key_action_is_save() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, TestCategory::General);
        });

        // When checking the action at path ['w'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the action is Save.
            assert_eq!(entries[0].action, TestAction::Save);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_third_key_count_is_one() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, TestCategory::Navigation);
        });

        // When checking bindings count.
        // Then there is exactly one binding (plus leader group).
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn scope_groups_binds_third_key_node_exists() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, TestCategory::Navigation);
        });

        // When looking up the node at path ['h'].
        let node =
            keymap.get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_third_key_entry_count_is_one() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, TestCategory::Navigation);
        });

        // When checking entry count at path ['h'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then there is exactly one entry.
            assert_eq!(entries.len(), 1);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_third_key_scope_is_global() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, TestCategory::Navigation);
        });

        // When checking the scope at path ['h'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the scope is Global.
            assert_eq!(entries[0].scope, TestScope::Global);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn scope_groups_binds_third_key_action_is_open() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<KeyEvent, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, TestCategory::Navigation);
        });

        // When checking the action at path ['h'].
        let node = keymap
            .get_node_at_path(&[KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty())])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the action is Open.
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn branch_preserves_children_when_adding_leaf_in_different_scope() {
        // Given bindings "<leader>gg" in Normal scope and "a" in Insert scope
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        keymap.bind(
            "<leader>gg",
            TestAction::Quit,
            TestCategory::Navigation,
            TestScope::Normal,
        );
        keymap.bind(
            "a",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );

        // Then navigating to 'a' in Normal scope still returns Branch
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Normal,
        );
        assert!(matches!(result, Some(NodeResult::Branch { .. })));

        // And navigating to 'a' in Insert scope returns Leaf
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty())],
            &TestScope::Insert,
        );
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Save
            })
        ));
    }

    #[test]
    fn leader_key_can_appear_in_sequence() {
        // Given leader='b' and binding "<leader>abc"
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> =
            Keymap::new().with_leader(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()));
        keymap.bind(
            "<leader>abc",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );

        // Then "babc" triggers the action
        let result = keymap.navigate(
            &[
                KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()),
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()),
            ],
            &TestScope::Global,
        );
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Quit
            })
        ));
    }

    #[test]
    fn same_key_leaf_in_one_scope_branch_in_another() {
        // Given "x" as leaf in Insert and "xyz" as sequence in Normal
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "x",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );
        keymap.bind(
            "xyz",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Normal,
        );

        // Then 'x' in Insert triggers Save
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())],
            &TestScope::Insert,
        );
        assert!(matches!(
            result,
            Some(NodeResult::Leaf {
                action: TestAction::Save
            })
        ));

        // And 'x' in Normal shows Branch (for xyz)
        let result = keymap.navigate(
            &[KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())],
            &TestScope::Normal,
        );
        assert!(matches!(result, Some(NodeResult::Branch { .. })));
    }

    #[test]
    fn describe_group_for_scope_same_prefix_shows_different_names_per_scope() {
        // Given a keymap with bindings "ta" in Global scope and "tb" in Insert scope,
        // each with a different group description for the "t" prefix.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "ta",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "tb",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );
        keymap.describe_group_for_scope("t", "t-one", TestScope::Global);
        keymap.describe_group_for_scope("t", "t-two", TestScope::Insert);

        // When getting bindings for Global scope.
        let global_bindings = keymap.get_bindings_for_scope(TestScope::Global);
        let t_binding_global = global_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Global scope");

        // Then the description is "t-one".
        assert_eq!(t_binding_global.description, "t-one");

        // When getting bindings for Insert scope.
        let insert_bindings = keymap.get_bindings_for_scope(TestScope::Insert);
        let t_binding_insert = insert_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Insert scope");

        // Then the description is "t-two".
        assert_eq!(t_binding_insert.description, "t-two");
    }

    #[test]
    fn describe_group_for_scope_via_scope_builder() {
        // Given a keymap using ScopeBuilder to set per-scope group descriptions.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.scope(TestScope::Global, |b| {
            b.describe_group("t", "t-one");
            b.bind("ta", TestAction::Quit, TestCategory::General);
        });
        keymap.scope(TestScope::Insert, |b| {
            b.describe_group("t", "t-two");
            b.bind("tb", TestAction::Save, TestCategory::General);
        });

        // When getting bindings for Global scope.
        let global_bindings = keymap.get_bindings_for_scope(TestScope::Global);
        let t_binding_global = global_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Global scope");

        // Then the description is "t-one".
        assert_eq!(t_binding_global.description, "t-one");

        // When getting bindings for Insert scope.
        let insert_bindings = keymap.get_bindings_for_scope(TestScope::Insert);
        let t_binding_insert = insert_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Insert scope");

        // Then the description is "t-two".
        assert_eq!(t_binding_insert.description, "t-two");
    }

    #[test]
    fn describe_group_for_scope_falls_back_to_default() {
        // Given a keymap with a default description and a scope-specific override.
        let mut keymap: Keymap<KeyEvent, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "ta",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "tb",
            TestAction::Save,
            TestCategory::General,
            TestScope::Insert,
        );
        keymap.describe_group("t", "default-name");
        keymap.describe_group_for_scope("t", "override-name", TestScope::Global);

        // When getting bindings for Global scope (has override).
        let global_bindings = keymap.get_bindings_for_scope(TestScope::Global);
        let t_global = global_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Global scope");

        // Then the scope-specific description takes priority.
        assert_eq!(t_global.description, "override-name");

        // When getting bindings for Insert scope (no override).
        let insert_bindings = keymap.get_bindings_for_scope(TestScope::Insert);
        let t_insert = insert_bindings
            .iter()
            .find(|b| b.key.display() == "t")
            .expect("Expected 't' binding in Insert scope");

        // Then the default description is used.
        assert_eq!(t_insert.description, "default-name");
    }
}
