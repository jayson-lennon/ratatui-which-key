use crate::{
    parse_key_sequence, Binding, BindingGroup, DisplayBinding, GroupBuilder, Key, KeyChild,
    KeyNode, LeafEntry, NodeResult, ScopeBuilder,
};
use tracing::warn;

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
            warn!(
                key = %first.display(),
                "Keybind creates group without description: key \"{}\" uses default \"...\" description. Use .describe_prefix() or .describe() to provide a group description.",
                first.display()
            );
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
                warn!(
                    key = %child.key.display(),
                    "Keybind creates group without description: key \"{}\" uses default \"...\" description. Use .describe_prefix() or .describe() to provide a group description.",
                    child.key.display()
                );
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
                .filter_map(|c| Self::find_category_in_children_recursive(&c.node, scope))
                .next()
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
                .filter_map(|c| Self::find_category_in_children_recursive(&c.node, scope))
                .next(),
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

    fn create_child_for_remaining_keys(
        keys: &[K],
        description: &'static str,
    ) -> KeyChild<K, S, A, C>
    where
        K: Clone,
        S: Clone,
        A: Clone,
        C: Clone,
    {
        if keys.len() == 1 {
            KeyChild::branch(keys[0].clone(), description, Vec::new())
        } else {
            Self::build_branch_tree(keys, description)
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
                let new_child = Self::create_child_for_remaining_keys(remaining, description);
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
                    let new_child = Self::create_child_for_remaining_keys(remaining, description);
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
    pub fn describe<F>(&mut self, prefix: &str, description: &'static str, bindings: F) -> &mut Self
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
}

impl<K: Key, S, A, C: Clone> Default for Keymap<K, S, A, C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn new_creates_empty_keymap_with_space_leader() {
        let keymap: Keymap<CrosstermKey, (), TestAction, TestCategory> = Keymap::new();

        assert_eq!(keymap.leader_key(), &CrosstermKey::Char(' '));
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn with_leader_creates_keymap_with_custom_leader() {
        let keymap: Keymap<CrosstermKey, (), TestAction, TestCategory> =
            Keymap::with_leader(CrosstermKey::Esc);

        assert_eq!(keymap.leader_key(), &CrosstermKey::Esc);
        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn default_creates_empty_keymap() {
        let keymap: Keymap<CrosstermKey, (), TestAction, TestCategory> = Keymap::default();

        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn bind_single_key_creates_leaf_node() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('q'));
        assert!(matches!(child.node, KeyNode::Leaf(_)));
    }

    #[test]
    fn bind_multi_key_sequence_creates_branch_structure() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('g'));
        assert!(child.node.is_branch());

        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].key, CrosstermKey::Char('g'));
            assert!(matches!(children[0].node, KeyNode::Leaf(_)));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_same_key_different_scopes_creates_multiple_entries() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
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

        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Esc);

        if let KeyNode::Leaf(entries) = &child.node {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].scope, TestScope::Global);
            assert_eq!(entries[1].scope, TestScope::Insert);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn bind_same_key_same_scope_updates_entry() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
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

        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];

        if let KeyNode::Leaf(entries) = &child.node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].action, TestAction::Save);
            assert_eq!(entries[0].description, "save");
            assert_eq!(entries[0].category, TestCategory::Navigation);
        } else {
            panic!("expected leaf node");
        }
    }

    #[test]
    fn bind_empty_sequence_does_nothing() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn bind_extends_existing_branch() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        assert_eq!(keymap.bindings().len(), 1);
        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('g'));

        if let KeyNode::Branch { children, .. } = &child.node {
            assert_eq!(children.len(), 2);
            let keys: Vec<_> = children.iter().map(|c| c.key.clone()).collect();
            assert!(keys.contains(&CrosstermKey::Char('g')));
            assert!(keys.contains(&CrosstermKey::Char('d')));
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn bind_converts_leaf_to_branch_when_extending() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "g",
            TestAction::Quit,
            "go",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.bind(
            "gg",
            TestAction::Open,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

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

        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn get_node_at_path_returns_none_for_empty_keys() {
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        let result = keymap.get_node_at_path(&[]);

        assert!(result.is_none());
    }

    #[test]
    fn get_node_at_path_returns_node_for_single_key() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        let result = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), KeyNode::Leaf(_)));
    }

    #[test]
    fn get_node_at_path_returns_node_for_multi_key_sequence() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let result = keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('g')]);

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), KeyNode::Leaf(_)));
    }

    #[test]
    fn get_node_at_path_returns_branch_for_prefix() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let result = keymap.get_node_at_path(&[CrosstermKey::Char('g')]);

        assert!(result.is_some());
        assert!(result.unwrap().is_branch());
    }

    #[test]
    fn get_node_at_path_returns_none_for_nonexistent_key() {
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        let result = keymap.get_node_at_path(&[CrosstermKey::Char('x')]);

        assert!(result.is_none());
    }

    #[test]
    fn get_children_at_path_returns_root_bindings_for_empty_keys() {
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

        let result = keymap.get_children_at_path(&[]);

        assert!(result.is_some());
        let children = result.unwrap();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn get_children_at_path_returns_children_for_branch() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.bind(
            "gd",
            TestAction::Open,
            "go to definition",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let result = keymap.get_children_at_path(&[CrosstermKey::Char('g')]);

        assert!(result.is_some());
        let children = result.unwrap();
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn get_children_at_path_returns_none_for_leaf() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        let result = keymap.get_children_at_path(&[CrosstermKey::Char('q')]);

        assert!(result.is_none());
    }

    #[test]
    fn get_children_at_path_returns_none_for_nonexistent_path() {
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        let result = keymap.get_children_at_path(&[CrosstermKey::Char('x')]);

        assert!(result.is_none());
    }

    #[test]
    fn is_prefix_key_returns_true_for_branch() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

        let result = keymap.is_prefix_key(CrosstermKey::Char('g'));

        assert!(result);
    }

    #[test]
    fn is_prefix_key_returns_false_for_leaf() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        let result = keymap.is_prefix_key(CrosstermKey::Char('q'));

        assert!(!result);
    }

    #[test]
    fn is_prefix_key_returns_false_for_nonexistent_key() {
        let keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        let result = keymap.is_prefix_key(CrosstermKey::Char('x'));

        assert!(!result);
    }

    #[test]
    fn get_bindings_for_scope_filters_by_exact_scope() {
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

        let result = keymap.get_bindings_for_scope(TestScope::Insert);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, CrosstermKey::Char('w'));
        assert_eq!(result[0].description, "save");
    }

    #[test]
    fn get_bindings_for_scope_returns_empty_for_no_matches() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        keymap.bind(
            "q",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );

        let result = keymap.get_bindings_for_scope(TestScope::Insert);

        assert!(result.is_empty());
    }

    #[test]
    fn get_bindings_for_scope_includes_branches() {
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

        let result = keymap.get_bindings_for_scope(TestScope::Global);

        assert_eq!(result.len(), 2);
        let keys: Vec<_> = result.iter().map(|b| b.key.clone()).collect();
        assert!(keys.contains(&CrosstermKey::Char('g')));
        assert!(keys.contains(&CrosstermKey::Char('q')));
    }

    #[test]
    fn describe_prefix_creates_branch_at_path() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.describe_prefix("g", "go commands");

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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.describe_prefix("g", "go commands");

        let child = &keymap.bindings()[0];
        if let KeyNode::Branch { description, .. } = &child.node {
            assert_eq!(*description, "go commands");
        } else {
            panic!("expected branch node");
        }
    }

    #[test]
    fn describe_prefix_works_for_nested_prefixes() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.describe_prefix("abc", "nested command");

        let child = &keymap.bindings()[0];
        assert_eq!(child.key, CrosstermKey::Char('a'));

        if let KeyNode::Branch {
            description,
            children,
        } = &child.node
        {
            assert_eq!(*description, "nested command");
            assert_eq!(children.len(), 1);
            assert_eq!(children[0].key, CrosstermKey::Char('b'));

            if let KeyNode::Branch {
                description,
                children,
            } = &children[0].node
            {
                assert_eq!(*description, "nested command");
                assert_eq!(children.len(), 1);
                assert_eq!(children[0].key, CrosstermKey::Char('c'));

                if let KeyNode::Branch {
                    description,
                    children,
                } = &children[0].node
                {
                    assert_eq!(*description, "nested command");
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
    fn describe_prefix_converts_leaf_to_branch() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.bind(
            "g",
            TestAction::Quit,
            "go",
            TestCategory::Navigation,
            TestScope::Global,
        );
        keymap.describe_prefix("g", "go commands");

        let child = &keymap.bindings()[0];
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
    fn describe_prefix_and_bind_work_together() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.describe_prefix("g", "go commands").bind(
            "gg",
            TestAction::Quit,
            "go to top",
            TestCategory::Navigation,
            TestScope::Global,
        );

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
    fn describe_prefix_returns_self_for_chaining() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap
            .describe_prefix("g", "go")
            .describe_prefix("d", "debug");

        assert_eq!(keymap.bindings().len(), 2);
    }

    #[test]
    fn describe_prefix_empty_string_does_nothing() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.describe_prefix("", "empty");

        assert!(keymap.bindings().is_empty());
    }

    #[test]
    fn scope_groups_multiple_bindings() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.scope(TestScope::Global, |b| {
            b.bind("q", TestAction::Quit, "quit", TestCategory::General)
                .bind("w", TestAction::Save, "save", TestCategory::General)
                .bind("h", TestAction::Open, "open", TestCategory::Navigation);
        });

        assert_eq!(keymap.bindings().len(), 3);

        let node_q = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);
        let node_w = keymap.get_node_at_path(&[CrosstermKey::Char('w')]);
        let node_h = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);

        assert!(node_q.is_some());
        assert!(node_w.is_some());
        assert!(node_h.is_some());

        if let Some(KeyNode::Leaf(entries)) = node_q {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].scope, TestScope::Global);
            assert_eq!(entries[0].action, TestAction::Quit);
        } else {
            panic!("expected leaf node for 'q'");
        }

        if let Some(KeyNode::Leaf(entries)) = node_w {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].scope, TestScope::Global);
            assert_eq!(entries[0].action, TestAction::Save);
        } else {
            panic!("expected leaf node for 'w'");
        }

        if let Some(KeyNode::Leaf(entries)) = node_h {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].scope, TestScope::Global);
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("expected leaf node for 'h'");
        }
    }

    #[test]
    fn describe_creates_group_with_description_and_bindings() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();

        keymap.describe("g", "goto", |b| {
            b.bind(
                "g",
                TestAction::Quit,
                "go to top",
                TestCategory::Navigation,
                TestScope::Global,
            )
            .bind(
                "e",
                TestAction::Save,
                "go to end",
                TestCategory::Navigation,
                TestScope::Global,
            );
        });

        let branch = keymap.get_node_at_path(&[CrosstermKey::Char('g')]);
        assert!(branch.is_some());
        assert!(branch.unwrap().is_branch());

        let node_gg = keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('g')]);
        assert!(node_gg.is_some());
        if let Some(KeyNode::Leaf(entries)) = node_gg {
            assert_eq!(entries[0].action, TestAction::Quit);
            assert_eq!(entries[0].description, "go to top");
        } else {
            panic!("expected leaf node for 'gg'");
        }

        let node_ge = keymap.get_node_at_path(&[CrosstermKey::Char('g'), CrosstermKey::Char('e')]);
        assert!(node_ge.is_some());
        if let Some(KeyNode::Leaf(entries)) = node_ge {
            assert_eq!(entries[0].action, TestAction::Save);
            assert_eq!(entries[0].description, "go to end");
        } else {
            panic!("expected leaf node for 'ge'");
        }
    }
}
