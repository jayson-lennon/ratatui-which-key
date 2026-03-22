use crate::{
    parse_key_sequence, Binding, BindingGroup, CategoryBuilder, DisplayBinding, GroupBuilder, Key,
    KeyChild, KeyNode, LeafEntry, NodeResult, ScopeAndCategoryBuilder, ScopeBuilder,
};
pub struct Keymap<K: Key, S, A, C> {
    bindings: Vec<KeyChild<K, S, A, C>>,
    leader_key: K,
}

impl<K: Key, S, A, C: Clone> Keymap<K, S, A, C> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            leader_key: K::space(),
        }
    }

    #[must_use]
    pub fn with_leader(leader_key: K) -> Self {
        Self {
            bindings: Vec::new(),
            leader_key,
        }
    }

    #[must_use]
    pub fn leader_key(&self) -> &K {
        &self.leader_key
    }

    #[must_use]
    pub fn bindings(&self) -> &[KeyChild<K, S, A, C>] {
        &self.bindings
    }

    pub fn bind(
        &mut self,
        sequence: &str,
        action: A,
        description: &'static str,
        category: C,
        scope: S,
    ) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        let keys = parse_key_sequence(sequence);
        if keys.is_empty() {
            return self;
        }
        self.insert_into_tree(&keys, action, description, category, scope);
        self
    }

    pub(super) fn insert_into_tree(
        &mut self,
        keys: &[K],
        action: A,
        description: &'static str,
        category: C,
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
            Self::insert_into_child(child, keys, action, description, category, scope);
        } else {
            let new_child = Self::build_tree(keys, action, description, category, scope);
            self.bindings.push(new_child);
        }
    }

    fn build_tree(
        keys: &[K],
        action: A,
        description: &'static str,
        category: C,
        scope: S,
    ) -> KeyChild<K, S, A, C>
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        if keys.len() == 1 {
            KeyChild::leaf(keys[0].clone(), action, description, category, scope)
        } else {
            let first = keys[0].clone();
            let rest = &keys[1..];
            let child = Self::build_tree(rest, action, description, category, scope);
            // warn!(
            //     key = %first.display(),
            //     "Keybind creates group without description: key \"{}\" uses default \"...\" description. Use .describe_prefix() or .describe() to provide a group description.",
            //     first.display()
            // );
            KeyChild::branch(first, "...", vec![child])
        }
    }

    fn insert_into_child(
        child: &mut KeyChild<K, S, A, C>,
        keys: &[K],
        action: A,
        description: &'static str,
        category: C,
        scope: S,
    ) where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        if keys.len() == 1 {
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
                KeyNode::Branch { .. } => {
                    child.node = KeyNode::Leaf(vec![LeafEntry {
                        action,
                        description,
                        category,
                        scope,
                    }]);
                }
            }
            return;
        }

        let next_key = keys[1].clone();
        match &mut child.node {
            KeyNode::Leaf(_) => {
                let new_child = Self::build_tree(&keys[1..], action, description, category, scope);
                // warn!(
                //     key = %child.key.display(),
                //     "Keybind creates group without description: key \"{}\" uses default \"...\" description. Use .describe_prefix() or .describe() to provide a group description.",
                //     child.key.display()
                // );
                child.node = KeyNode::Branch {
                    description: "...",
                    children: vec![new_child],
                };
            }
            KeyNode::Branch { children, .. } => {
                if let Some(next_child) = children.iter_mut().find(|c| c.key == next_key) {
                    Self::insert_into_child(
                        next_child,
                        &keys[1..],
                        action,
                        description,
                        category,
                        scope,
                    );
                } else {
                    let new_child =
                        Self::build_tree(&keys[1..], action, description, category, scope);
                    children.push(new_child);
                }
            }
        }
    }

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

    #[must_use]
    pub fn get_children_at_path(&self, keys: &[K]) -> Option<Vec<(K, &'static str)>>
    where
        K: PartialEq + Clone,
    {
        let node = if keys.is_empty() {
            return Some(
                self.bindings
                    .iter()
                    .map(|c| (c.key.clone(), c.node.description()))
                    .collect(),
            );
        } else {
            self.get_node_at_path(keys)?
        };

        match node {
            KeyNode::Branch { children, .. } => Some(
                children
                    .iter()
                    .map(|c| (c.key.clone(), c.node.description()))
                    .collect(),
            ),
            KeyNode::Leaf(_) => None,
        }
    }

    #[must_use]
    pub fn is_prefix_key(&self, key: K) -> bool
    where
        K: PartialEq,
    {
        self.bindings
            .iter()
            .any(|c| c.key == key && c.node.is_branch())
    }

    #[must_use]
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
                            description: entry.description,
                            category: entry.category.clone(),
                        })
                }
                KeyNode::Branch { description, .. } => {
                    Self::find_category_in_children(&child.node, &scope).map(|category| {
                        DisplayBinding {
                            key: child.key.clone(),
                            description,
                            category,
                        }
                    })
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
                .map(|e| e.category.clone())
                .or_else(|| entries.first().map(|e| e.category.clone())),
            KeyNode::Branch { children, .. } => children
                .iter()
                .find_map(|c| Self::find_category_in_children_recursive(&c.node, scope))
                .or_else(|| {
                    children
                        .first()
                        .and_then(|c| Self::find_category_in_children(&c.node, scope))
                }),
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

    #[must_use]
    pub fn bindings_for_scope(&self, scope: S) -> Vec<BindingGroup<K>>
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone + std::fmt::Debug,
    {
        let bindings = self.get_bindings_for_scope(scope);
        let groups: std::collections::BTreeMap<String, Vec<Binding<K>>> = bindings
            .iter()
            .map(|b| {
                (
                    format!("{:?}", b.category),
                    Binding {
                        key: b.key.clone(),
                        description: b.description,
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

    #[must_use]
    pub fn children_at_path(&self, keys: &[K]) -> Option<Vec<Binding<K>>>
    where
        K: Clone + PartialEq,
    {
        self.get_children_at_path(keys).map(|children| {
            children
                .into_iter()
                .map(|(key, description)| Binding { key, description })
                .collect()
        })
    }

    #[must_use]
    pub fn navigate(&self, keys: &[K]) -> Option<NodeResult<K, A>>
    where
        K: Clone + PartialEq,
        A: Clone,
    {
        let node = self.get_node_at_path(keys)?;
        match node {
            KeyNode::Branch { children, .. } => Some(NodeResult::Branch {
                children: children
                    .iter()
                    .map(|c| Binding {
                        key: c.key.clone(),
                        description: c.node.description(),
                    })
                    .collect(),
            }),
            KeyNode::Leaf(entries) => entries.first().map(|e| NodeResult::Leaf {
                action: e.action.clone(),
            }),
        }
    }

    pub fn describe_group(&mut self, prefix: &str, description: &'static str) -> &mut Self
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
        self.ensure_branch_with_description(&keys, description);
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
                    children: Vec::new(),
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
                    children: vec![new_child],
                };
            }
            KeyNode::Branch {
                description: desc,
                children,
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

    /// Creates a group of bindings under a prefix with a description.
    ///
    /// # Example
    ///
    /// ```ignore
    /// keymap.describe("g", "goto", |b| {
    ///     b.bind("g", Action::GoTop, "go to top", Navigation)
    ///          .bind("e", Action::GoEnd, "go to end", Navigation);
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
        let prefix_keys = parse_key_sequence(prefix);
        if prefix_keys.is_empty() {
            return self;
        }
        self.ensure_branch_with_description(&prefix_keys, description);
        let mut builder = GroupBuilder::new(self, prefix_keys);
        bindings(&mut builder);
        self
    }

    pub fn scope<F>(&mut self, scope: S, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut ScopeBuilder<K, S, A, C>),
    {
        let mut builder = ScopeBuilder::new(self, scope);
        bindings(&mut builder);
        self
    }

    pub fn category<F>(&mut self, category: C, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut CategoryBuilder<'_, K, S, A, C>),
    {
        let mut builder = CategoryBuilder::new(self, category);
        bindings(&mut builder);
        self
    }

    pub fn scope_and_category<F>(&mut self, scope: S, category: C, bindings: F) -> &mut Self
    where
        F: FnOnce(&mut ScopeAndCategoryBuilder<'_, K, S, A, C>),
    {
        let mut builder = ScopeAndCategoryBuilder::new(self, scope, category);
        bindings(&mut builder);
        self
    }
}

impl<K: Key, S, A, C: Clone> Default for Keymap<K, S, A, C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::keymap_with_binding;
    use crate::CrosstermKey;

    #[derive(Debug, Clone, PartialEq)]
    enum TestAction {
        Quit,
        Save,
        Open,
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
        keymap: &Keymap<CrosstermKey, S, A, C>,
    ) -> Vec<(S, A, C, String)> {
        let child = &keymap.bindings()[0];
        if let KeyNode::Leaf(entries) = &child.node {
            entries
                .iter()
                .map(|e| {
                    (
                        e.scope.clone(),
                        e.action.clone(),
                        e.category.clone(),
                        e.description.to_string(),
                    )
                })
                .collect()
        } else {
            panic!("expected leaf node");
        }
    }

    fn get_leaf_entry_count<S: Clone + PartialEq, A: Clone, C: Clone + PartialEq>(
        keymap: &Keymap<CrosstermKey, S, A, C>,
    ) -> usize {
        let child = &keymap.bindings()[0];
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
        let keymap: Keymap<CrosstermKey, (), TestAction, TestCategory> = Keymap::new();

        // Then it has a space leader key and no bindings.
        assert_eq!(keymap.leader_key(), &CrosstermKey::Char(' '));
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn with_leader_creates_keymap_with_custom_leader() {
        // Given no key bindings.

        // When creating a keymap with a custom leader key.
        let keymap: Keymap<CrosstermKey, (), TestAction, TestCategory> =
            Keymap::with_leader(CrosstermKey::Esc);

        // Then it uses the custom leader and has no bindings.
        assert_eq!(keymap.leader_key(), &CrosstermKey::Esc);
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn bind_single_key_creates_leaf_node() {
        // Given a keymap with a single binding.
        let keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        // Then a leaf node is created with the binding.
        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('q'));
        assert!(matches!(child.node, KeyNode::Leaf(_)));
    }

    #[test]
    fn multi_key_binding_count_is_one() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then there is exactly one binding.
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn multi_key_root_key_is_g() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root key is 'g'.
        assert_eq!(keymap.bindings()[0].key, CrosstermKey::Char('g'));
    }

    #[test]
    fn multi_key_root_is_branch() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root node is a branch.
        assert!(keymap.bindings()[0].node.is_branch());
    }

    #[test]
    fn multi_key_second_level_count_is_one() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch has one child.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 1);
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn multi_key_second_level_key_is_g() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the second level key is 'g'.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children[0].key, CrosstermKey::Char('g'));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn multi_key_second_level_is_leaf() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding a multi-key sequence.
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the second level node is a leaf.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            assert!(matches!(children[0].node, KeyNode::Leaf(_)));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn same_scope_multi_entry_count_is_two() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            "save and quit",
            TestCategory::General,
            TestScope::Insert,
        );

        let count = get_leaf_entry_count(&keymap);

        assert_eq!(count, 2);
    }

    #[test]
    fn same_scope_multi_entry_first_scope_is_global() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            "save and quit",
            TestCategory::General,
            TestScope::Insert,
        );

        let entries = get_leaf_entries(&keymap);

        assert_eq!(entries[0].0, TestScope::Global);
    }

    #[test]
    fn same_scope_multi_entry_second_scope_is_insert() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "<esc>",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "<esc>",
            TestAction::Save,
            "save and quit",
            TestCategory::General,
            TestScope::Insert,
        );

        let entries = get_leaf_entries(&keymap);

        assert_eq!(entries[1].0, TestScope::Insert);
    }

    #[test]
    fn same_scope_update_entry_count_is_one() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            "save",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let count = get_leaf_entry_count(&keymap);

        assert_eq!(count, 1);
    }

    #[test]
    fn same_scope_update_action_is_save() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            "save",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(&keymap);

        assert_eq!(entries[0].1, TestAction::Save);
    }

    #[test]
    fn same_scope_update_description_is_save() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            "save",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(&keymap);

        assert_eq!(entries[0].3, "save");
    }

    #[test]
    fn same_scope_update_category_is_navigation() {
        let mut keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        keymap.bind(
            "q",
            TestAction::Save,
            "save",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let entries = get_leaf_entries(&keymap);

        assert_eq!(entries[0].2, TestCategory::Navigation);
    }

    #[test]
    fn bind_empty_sequence_does_nothing() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When binding an empty sequence.
        keymap.bind(
            "",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        // Then nothing is bound.
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn branch_extension_binding_count_is_one() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then there is still only one binding at the root.
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn branch_extension_root_key_is_g() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the root key is 'g'.
        assert_eq!(keymap.bindings()[0].key, CrosstermKey::Char('g'));
    }

    #[test]
    fn branch_extension_children_count_is_two() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch has two children.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 2);
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn branch_extension_includes_d_key() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch includes the 'd' key.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            let keys: Vec<_> = children.iter().map(|c| c.key.clone()).collect();
            assert!(keys.contains(&CrosstermKey::Char('d')));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn branch_extension_preserves_g_key() {
        // Given a keymap with a "gg" binding.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding "gd" which extends the same branch.
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the branch still includes the 'g' key.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { children, .. } = &child.node {
            let keys: Vec<_> = children.iter().map(|c| c.key.clone()).collect();
            assert!(keys.contains(&CrosstermKey::Char('g')));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_converts_leaf_to_branch_when_extending() {
        // Given a keymap with a leaf node.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "g",
            TestAction::Quit,
            "go",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When binding a multi-key sequence starting with that key.
        keymap.bind(
            "gg",
            TestAction::Open,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the leaf is converted to a branch.
        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert!(child.node.is_branch());

        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].key, CrosstermKey::Char('g'));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_returns_self_for_chaining() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When chaining multiple bind calls.
        keymap
            .bind(
                "q",
                TestAction::Quit,
                "quit",
                TestCategory::General,
                TestScope::Global,
            )
            .bind(
                "w",
                TestAction::Save,
                "save",
                TestCategory::General,
                TestScope::Global,
            );

        // Then both bindings are added.
        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn get_node_at_path_returns_none_for_empty_keys() {
        // Given an empty keymap.
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When getting a node with an empty path.
        let result = keymap.get_node_at_path(&[]);

        // Then no node is returned.
        assert!(result.is_none());
    }

    #[test]
    fn get_node_at_path_returns_none_for_nonexistent_key() {
        // Given an empty keymap.
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When getting a node with a nonexistent key.
        let result = keymap.get_node_at_path(&[CrosstermKey::Char('x')]);

        // Then no node is returned.
        assert!(result.is_none());
    }

    #[test]
    fn get_children_at_path_returns_root_bindings_for_empty_keys() {
        // Given a keymap with multiple bindings.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "w",
            TestAction::Save,
            "save",
            TestCategory::General,
            TestScope::Global,
        );

        // When getting children at an empty path.
        let result = keymap.get_children_at_path(&[]);

        // Then root bindings are returned.
        assert!(result.is_some());
        let children = result.unwrap();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn get_children_at_path_returns_none_for_leaf() {
        // Given a keymap with a leaf node.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        // When getting children at the leaf path.
        let result = keymap.get_children_at_path(&[CrosstermKey::Char('q')]);

        // Then none is returned.
        assert!(result.is_none());
    }

    #[test]
    fn is_prefix_key_returns_false_for_leaf() {
        // Given a keymap with a leaf node.
        let keymap = keymap_with_binding::<CrosstermKey, TestScope, TestAction, TestCategory>(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        // When checking if 'q' is a prefix key.
        let result = keymap.is_prefix_key(CrosstermKey::Char('q'));

        // Then it returns false.
        assert!(!result);
    }

    #[test]
    fn get_bindings_for_scope_filters_by_exact_scope() {
        // Given a keymap with bindings in different scopes.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );
        keymap.bind(
            "w",
            TestAction::Save,
            "save",
            TestCategory::General,
            TestScope::Insert,
        );
        keymap.bind(
            "e",
            TestAction::Open,
            "open",
            TestCategory::General,
            TestScope::Normal,
        );

        // When getting bindings for a specific scope.
        let result = keymap.get_bindings_for_scope(TestScope::Insert);

        // Then only bindings with that exact scope are returned.
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, CrosstermKey::Char('w'));
        assert_eq!(result[0].description, "save");
    }

    #[test]
    fn get_bindings_for_scope_returns_empty_for_no_matches() {
        // Given a keymap with bindings in Global scope.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        // When getting bindings for Global scope.
        let result = keymap.get_bindings_for_scope(TestScope::Global);

        // Then both branch and leaf bindings are included.
        assert_eq!(result.len(), 2);
        let keys: Vec<_> = result.iter().map(|b| b.key.clone()).collect();
        assert!(keys.contains(&CrosstermKey::Char('g')));
        assert!(keys.contains(&CrosstermKey::Char('q')));
    }

    #[test]
    fn describe_prefix_creates_branch_at_path() {
        // Given an empty keymap.
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        // When describing a prefix.
        keymap.describe_group("g", "go commands");

        // Then a branch is created with the description.
        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('g'));
        assert!(child.node.is_branch());

        if let KeyNode::Branch {
            description,
            children,
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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // When describing the prefix.
        keymap.describe_group("g", "go commands");

        // Then the description is updated.
        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { description, .. } = &child.node {
            assert_eq!(*description, "go commands");
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn single_level_branch_count_is_one() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn single_level_branch_key_is_a() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        assert_eq!(keymap.bindings()[0].key, CrosstermKey::Char('a'));
    }

    #[test]
    fn single_level_branch_description_is_set() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        if let KeyNode::Branch { description, .. } = &keymap.bindings()[0].node {
            assert_eq!(*description, "single command");
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn single_level_branch_has_no_children() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("a", "single command");

        if let KeyNode::Branch { children, .. } = &keymap.bindings()[0].node {
            assert!(children.is_empty());
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn nested_branches_first_key_is_a() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        assert_eq!(keymap.bindings()[0].key, CrosstermKey::Char('a'));
    }

    #[test]
    fn nested_branches_first_description_is_nested_command() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        if let KeyNode::Branch { description, .. } = &keymap.bindings()[0].node {
            assert_eq!(*description, "nested command");
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_second_key_is_b() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        if let KeyNode::Branch { children, .. } = &keymap.bindings()[0].node {
            assert_eq!(children[0].key, CrosstermKey::Char('b'));
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_third_key_is_c() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        if let KeyNode::Branch { children, .. } = &keymap.bindings()[0].node {
            if let KeyNode::Branch { children, .. } = &children[0].node {
                assert_eq!(children[0].key, CrosstermKey::Char('c'));
            } else {
                panic!("expected branch node at 'b'");
            }
        } else {
            panic!("expected branch node at 'a'");
        }
    }

    #[test]
    fn nested_branches_third_level_is_leaf() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("abc", "nested command");

        if let KeyNode::Branch { children, .. } = &keymap.bindings()[0].node {
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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.describe_group("g", "go commands").bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        // Then the prefix is described and the binding is under it.
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('g'));

        if let KeyNode::Branch {
            description,
            children,
        } = &child.node
        {
            assert_eq!(*description, "go commands");
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].key, CrosstermKey::Char('g'));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn describe_prefix_empty_string_does_nothing() {
        // Given an empty keymap.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();

        // When describing an empty prefix.
        keymap.describe_group("", "empty");

        // Then no bindings are created.
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn scope_groups_binds_first_key_count_is_one() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General);
        });

        // When checking bindings count.
        // Then there is exactly one binding.
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn scope_groups_binds_first_key_node_exists() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General);
        });

        // When looking up the node at path ['q'].
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_first_key_entry_count_is_one() {
        // Given a keymap with a global 'q' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General);
        });

        // When checking entry count at path ['q'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('q')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General);
        });

        // When checking the scope at path ['q'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('q')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General);
        });

        // When checking the action at path ['q'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('q')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, "save", TestCategory::General);
        });

        // When checking bindings count.
        // Then there is exactly one binding.
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn scope_groups_binds_second_key_node_exists() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, "save", TestCategory::General);
        });

        // When looking up the node at path ['w'].
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('w')]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_second_key_entry_count_is_one() {
        // Given a keymap with a global 'w' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, "save", TestCategory::General);
        });

        // When checking entry count at path ['w'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('w')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, "save", TestCategory::General);
        });

        // When checking the scope at path ['w'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('w')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("w", TestAction::Save, "save", TestCategory::General);
        });

        // When checking the action at path ['w'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('w')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        // When checking bindings count.
        // Then there is exactly one binding.
        assert_eq!(keymap.bindings().len(), 1);
    }

    #[test]
    fn scope_groups_binds_third_key_node_exists() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        // When looking up the node at path ['h'].
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);

        // Then the node exists.
        assert!(node.is_some());
    }

    #[test]
    fn scope_groups_binds_third_key_entry_count_is_one() {
        // Given a keymap with a global 'h' binding.
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        // When checking entry count at path ['h'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('h')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        // When checking the scope at path ['h'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('h')])
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
        let mut keymap = Keymap::<CrosstermKey, TestScope, TestAction, TestCategory>::new();
        keymap.scope(TestScope::Global, |b| {
            b.bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        // When checking the action at path ['h'].
        let node = keymap
            .get_node_at_path(&[CrosstermKey::Char('h')])
            .expect("node exists");
        if let KeyNode::Leaf(entries) = node {
            // Then the action is Open.
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("expected leaf node");
        }
    }
}
