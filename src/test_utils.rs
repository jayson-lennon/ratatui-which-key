#![allow(dead_code)]
#[cfg(test)]
use crate::Key;
#[cfg(test)]
use crate::{Keymap, WhichKeyState};

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKey {
    Char(char),
    Backspace,
}

#[cfg(test)]
impl Key for TestKey {
    fn display(&self) -> String {
        match self {
            TestKey::Char(c) => c.to_string(),
            TestKey::Backspace => "BS".to_string(),
        }
    }

    fn is_backspace(&self) -> bool {
        matches!(self, TestKey::Backspace)
    }

    fn from_char(c: char) -> Option<Self> {
        Some(TestKey::Char(c))
    }

    fn space() -> Self {
        TestKey::Char(' ')
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub enum TestAction {
    Quit,
    Save,
    Open,
}

#[cfg(test)]
impl std::fmt::Display for TestAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestAction::Quit => write!(f, "Quit"),
            TestAction::Save => write!(f, "Save"),
            TestAction::Open => write!(f, "Open"),
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub enum TestScope {
    Global,
    Insert,
    Normal,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    General,
    Navigation,
}

#[cfg(test)]
pub fn state_with_pending_keys<K, S, A, C>(
    keymap: Keymap<K, S, A, C>,
    keys: &[K],
    scope: S,
) -> WhichKeyState<K, S, A, C>
where
    K: crate::Key + Clone + PartialEq,
    S: Clone + PartialEq + Send + Sync,
    A: Clone + Send + Sync,
    C: Clone + std::fmt::Debug,
{
    let mut state = WhichKeyState::new(keymap, scope);
    state.active = true;
    state.current_sequence.extend_from_slice(keys);
    state
}

#[cfg(test)]
pub fn state_with_binding_and_sequence<K, S, A, C>(
    key_sequence: &str,
    action: A,
    cat: C,
    scope: S,
    pending_keys: &[K],
) -> WhichKeyState<K, S, A, C>
where
    K: crate::Key + Clone,
    S: Clone + PartialEq + Send + Sync,
    A: Clone + Send + Sync + std::fmt::Display,
    C: Clone + PartialEq + std::fmt::Debug,
{
    let mut keymap = Keymap::new();
    keymap.bind(key_sequence, action, cat, scope.clone());
    state_with_pending_keys(keymap, pending_keys, scope)
}

#[cfg(test)]
use crate::{CrosstermKey, KeyNode};

#[cfg(test)]
pub fn assert_leaf_entry<K, S, A, C>(
    node: Option<&KeyNode<K, S, A, C>>,
    expected_action: A,
    expected_scope: S,
) where
    K: crate::Key + std::fmt::Debug,
    S: Clone + PartialEq + std::fmt::Debug,
    A: Clone + PartialEq + std::fmt::Debug,
    C: Clone,
{
    let node = node.expect("Expected Some node, got None");
    let entries = match node {
        KeyNode::Leaf(entries) => entries,
        KeyNode::Branch { .. } => panic!("Expected Leaf node, got Branch"),
    };
    assert_eq!(
        entries.len(),
        1,
        "Expected exactly 1 entry, found {}",
        entries.len()
    );
    assert_eq!(entries[0].action, expected_action, "Action mismatch");
    assert_eq!(entries[0].scope, expected_scope, "Scope mismatch");
}

#[cfg(test)]
pub fn assert_branch_at_path<K, S, A, C>(
    keymap: &Keymap<K, S, A, C>,
    path: &[K],
    expected_description: &str,
) where
    K: crate::Key + PartialEq,
    S: Clone,
    A: Clone,
    C: Clone,
{
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    match node {
        KeyNode::Leaf(_) => panic!("Expected Branch node at path, got Leaf"),
        KeyNode::Branch {
            description,
            children,
        } => {
            assert_eq!(*description, expected_description, "Description mismatch");
            assert!(
                children.is_empty(),
                "Expected no children (placeholder), found {}",
                children.len()
            );
        }
    }
}

#[cfg(test)]
pub fn assert_nth_child_is_branch<K, S, A, C>(
    parent: &KeyNode<K, S, A, C>,
    index: usize,
    expected_key: K,
) -> &KeyNode<K, S, A, C>
where
    K: crate::Key + PartialEq + std::fmt::Debug,
    S: Clone,
    A: Clone,
    C: Clone,
{
    let children = match parent {
        KeyNode::Branch { children, .. } => children,
        KeyNode::Leaf(_) => panic!("Expected Branch node, got Leaf"),
    };
    assert!(
        index < children.len(),
        "Index {} out of bounds (children count: {})",
        index,
        children.len()
    );
    let child = &children[index];
    assert_eq!(
        child.key, expected_key,
        "Child key mismatch at index {index}",
    );
    assert!(
        child.node.is_branch(),
        "Expected child at index {index} to be a Branch",
    );
    &child.node
}

#[cfg(test)]
pub fn assert_nth_child_is_leaf<K, S, A, C>(
    parent: &KeyNode<K, S, A, C>,
    index: usize,
    expected_key: K,
    expected_action: A,
    expected_scope: S,
) where
    K: crate::Key + PartialEq + std::fmt::Debug,
    S: Clone + PartialEq + std::fmt::Debug,
    A: Clone + PartialEq + std::fmt::Debug,
    C: Clone,
{
    let children = match parent {
        KeyNode::Branch { children, .. } => children,
        KeyNode::Leaf(_) => panic!("Expected Branch node, got Leaf"),
    };
    assert!(
        index < children.len(),
        "Index {} out of bounds (children count: {})",
        index,
        children.len()
    );
    let child = &children[index];
    assert_eq!(
        child.key, expected_key,
        "Child key mismatch at index {index}",
    );
    match &child.node {
        KeyNode::Leaf(entries) => {
            assert_eq!(
                entries.len(),
                1,
                "Expected exactly 1 entry, found {}",
                entries.len()
            );
            assert_eq!(entries[0].action, expected_action, "Action mismatch");
            assert_eq!(entries[0].scope, expected_scope, "Scope mismatch");
        }
        KeyNode::Branch { .. } => panic!("Expected Leaf at index {index}, got Branch"),
    }
}

#[cfg(test)]
pub fn keymap_with_binding<K, S, A, C>(
    keys: &str,
    action: A,
    cat: C,
    scope: S,
) -> Keymap<K, S, A, C>
where
    K: crate::Key,
    S: Clone + PartialEq,
    A: Clone + std::fmt::Display,
    C: Clone + PartialEq,
{
    let mut keymap = Keymap::new();
    keymap.bind(keys, action, cat, scope);
    keymap
}

#[cfg(test)]
pub fn test_keymap_with_binding(
    key: &str,
    action: TestAction,
    category: TestCategory,
    scope: TestScope,
) -> Keymap<CrosstermKey, TestScope, TestAction, TestCategory> {
    let mut keymap = Keymap::new();
    keymap.bind(key, action, category, scope);
    keymap
}

#[cfg(test)]
pub fn test_keymap_with_scope_binding(
    key: &str,
    action: TestAction,
    category: TestCategory,
    scope: TestScope,
) -> Keymap<CrosstermKey, TestScope, TestAction, TestCategory> {
    let mut keymap = Keymap::new();
    keymap.scope(scope, |b| {
        b.bind(key, action, category);
    });
    keymap
}

#[cfg(test)]
pub fn assert_leaf_key_and_action(
    keymap: &Keymap<CrosstermKey, TestScope, TestAction, TestCategory>,
    path: &[CrosstermKey],
    expected_action: TestAction,
) {
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    let entries = match node {
        KeyNode::Leaf(entries) => entries,
        KeyNode::Branch { .. } => panic!("Expected Leaf node at path, got Branch"),
    };
    assert_eq!(entries[0].action, expected_action);
}

#[cfg(test)]
pub fn assert_leaf_entry_count(
    keymap: &Keymap<CrosstermKey, TestScope, TestAction, TestCategory>,
    path: &[CrosstermKey],
    expected_count: usize,
) {
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    let entries = match node {
        KeyNode::Leaf(entries) => entries,
        KeyNode::Branch { .. } => panic!("Expected Leaf node at path, got Branch"),
    };
    assert_eq!(entries.len(), expected_count);
}

#[cfg(test)]
pub fn assert_leaf_scope_at_index(
    keymap: &Keymap<CrosstermKey, TestScope, TestAction, TestCategory>,
    path: &[CrosstermKey],
    index: usize,
    expected_scope: TestScope,
) {
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    let entries = match node {
        KeyNode::Leaf(entries) => entries,
        KeyNode::Branch { .. } => panic!("Expected Leaf node at path, got Branch"),
    };
    assert_eq!(entries[index].scope, expected_scope);
}

#[cfg(test)]
pub fn assert_branch_child_key(
    keymap: &Keymap<CrosstermKey, TestScope, TestAction, TestCategory>,
    path: &[CrosstermKey],
    child_index: usize,
    expected_key: CrosstermKey,
) {
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    let children = match node {
        KeyNode::Leaf(_) => panic!("Expected Branch node at path, got Leaf"),
        KeyNode::Branch { children, .. } => children,
    };
    assert_eq!(children[child_index].key, expected_key);
}

#[cfg(test)]
pub fn assert_branch_description(
    keymap: &Keymap<CrosstermKey, TestScope, TestAction, TestCategory>,
    path: &[CrosstermKey],
    expected_description: &str,
) {
    let node = keymap
        .get_node_at_path(path)
        .expect("Expected node at path");
    let description = match node {
        KeyNode::Leaf(_) => panic!("Expected Branch node at path, got Leaf"),
        KeyNode::Branch { description, .. } => *description,
    };
    assert_eq!(description, expected_description);
}
