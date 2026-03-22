use crate::Key;

#[derive(Debug, Clone)]
pub struct LeafEntry<S, A, C> {
    pub action: A,
    pub description: &'static str,
    pub category: C,
    pub scope: S,
}

#[derive(Debug, Clone)]
pub enum KeyNode<K: Key, S, A, C> {
    Leaf(Vec<LeafEntry<S, A, C>>),
    Branch {
        description: &'static str,
        children: Vec<KeyChild<K, S, A, C>>,
    },
}

impl<K: Key, S, A, C> KeyNode<K, S, A, C> {
    pub fn description(&self) -> &'static str {
        match self {
            KeyNode::Leaf(entries) => entries.first().map_or("", |e| e.description),
            KeyNode::Branch { description, .. } => description,
        }
    }

    pub fn is_branch(&self) -> bool {
        matches!(self, KeyNode::Branch { .. })
    }

    pub fn category(&self) -> Option<C>
    where
        C: Clone,
    {
        match self {
            KeyNode::Leaf(entries) => entries.first().map(|e| e.category.clone()),
            KeyNode::Branch { .. } => None,
        }
    }

    pub fn find_child_mut(&mut self, key: &K) -> Option<&mut KeyChild<K, S, A, C>>
    where
        K: PartialEq,
    {
        match self {
            KeyNode::Leaf(_) => None,
            KeyNode::Branch { children, .. } => children.iter_mut().find(|c| c.key == *key),
        }
    }

    pub fn find_child(&self, key: &K) -> Option<&KeyChild<K, S, A, C>>
    where
        K: PartialEq,
    {
        match self {
            KeyNode::Leaf(_) => None,
            KeyNode::Branch { children, .. } => children.iter().find(|c| c.key == *key),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyChild<K: Key, S, A, C> {
    pub key: K,
    pub node: KeyNode<K, S, A, C>,
}

impl<K: Key, S: Clone, A: Clone, C: Clone> KeyChild<K, S, A, C> {
    pub fn new(key: K, node: KeyNode<K, S, A, C>) -> Self {
        Self { key, node }
    }

    pub fn leaf(key: K, action: A, description: &'static str, category: C, scope: S) -> Self {
        let entry = LeafEntry {
            action,
            description,
            category,
            scope,
        };
        Self {
            key,
            node: KeyNode::Leaf(vec![entry]),
        }
    }

    pub fn branch(key: K, description: &'static str, children: Vec<Self>) -> Self {
        Self {
            key,
            node: KeyNode::Branch {
                description,
                children,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LeafBinding<K: Key, S, A, C> {
    pub key: K,
    pub action: A,
    pub description: &'static str,
    pub category: C,
    pub scope: S,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{TestAction, TestCategory, TestKey, TestScope};

    fn create_test_leaf() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        KeyChild::leaf(
            TestKey::Char('q'),
            TestAction::Quit,
            "quit application",
            TestCategory::General,
            TestScope::Normal,
        )
    }

    fn create_test_branch() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        let save_child = KeyChild::leaf(
            TestKey::Char('s'),
            TestAction::Save,
            "save file",
            TestCategory::General,
            TestScope::Normal,
        );
        let open_child = KeyChild::leaf(
            TestKey::Char('o'),
            TestAction::Open,
            "open file",
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
        let child = create_test_leaf();

        assert!(!child.node.is_branch());
    }

    #[test]
    fn key_child_branch_creates_branch_node() {
        let child = create_test_branch();

        assert!(child.node.is_branch());
    }

    #[test]
    fn key_node_description_returns_leaf_description() {
        let child = create_test_leaf();

        assert_eq!(child.node.description(), "quit application");
    }

    #[test]
    fn key_node_description_returns_branch_description() {
        let child = create_test_branch();

        assert_eq!(child.node.description(), "file operations");
    }

    #[test]
    fn key_node_category_returns_leaf_category() {
        let child = create_test_leaf();

        assert_eq!(child.node.category(), Some(TestCategory::General));
    }

    #[test]
    fn key_node_category_returns_none_for_branch() {
        let child = create_test_branch();

        assert_eq!(child.node.category(), None);
    }

    #[test]
    fn key_child_new_creates_child_with_node() {
        let node = KeyNode::Leaf(vec![LeafEntry {
            action: TestAction::Quit,
            description: "quit",
            category: TestCategory::General,
            scope: TestScope::Normal,
        }]);

        let child = KeyChild::new(TestKey::Char('q'), node.clone());

        assert_eq!(child.key, TestKey::Char('q'));
        assert!(matches!(child.node, KeyNode::Leaf(_)));
    }

    #[test]
    fn leaf_entry_stores_all_fields() {
        let entry = LeafEntry {
            action: TestAction::Save,
            description: "save file",
            category: TestCategory::Navigation,
            scope: TestScope::Insert,
        };

        assert_eq!(entry.action, TestAction::Save);
        assert_eq!(entry.description, "save file");
        assert_eq!(entry.category, TestCategory::Navigation);
        assert_eq!(entry.scope, TestScope::Insert);
    }

    #[test]
    fn leaf_binding_stores_all_fields() {
        let binding = LeafBinding {
            key: TestKey::Char('w'),
            action: TestAction::Save,
            description: "write file",
            category: TestCategory::General,
            scope: TestScope::Normal,
        };

        assert_eq!(binding.key, TestKey::Char('w'));
        assert_eq!(binding.action, TestAction::Save);
        assert_eq!(binding.description, "write file");
        assert_eq!(binding.category, TestCategory::General);
        assert_eq!(binding.scope, TestScope::Normal);
    }

    #[test]
    fn branch_with_empty_children() {
        let child: KeyChild<TestKey, TestScope, TestAction, TestCategory> =
            KeyChild::branch(TestKey::Char('x'), "empty branch", vec![]);

        assert!(child.node.is_branch());
        assert_eq!(child.node.description(), "empty branch");
    }

    #[test]
    fn nested_branch_structure() {
        let inner_leaf = KeyChild::leaf(
            TestKey::Char('d'),
            TestAction::Quit,
            "delete",
            TestCategory::General,
            TestScope::Normal,
        );
        let inner_branch =
            KeyChild::branch(TestKey::Char('d'), "delete operations", vec![inner_leaf]);
        let outer_branch = KeyChild::branch(TestKey::Char('g'), "go to", vec![inner_branch]);

        assert!(outer_branch.node.is_branch());

        if let KeyNode::Branch { children, .. } = outer_branch.node {
            assert_eq!(children.len(), 1);
            assert!(children[0].node.is_branch());
        } else {
            panic!("expected branch node");
        }
    }
}
