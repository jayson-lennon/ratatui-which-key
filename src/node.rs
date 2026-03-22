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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestCategory {
        General,
        Navigation,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestKey(char);

    impl Key for TestKey {
        fn display(&self) -> String {
            self.0.to_string()
        }

        fn is_escape(&self) -> bool {
            self.0 == '\x1b'
        }

        fn is_backspace(&self) -> bool {
            self.0 == '\x7f'
        }

        fn space() -> Self {
            TestKey(' ')
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestAction {
        Quit,
        Save,
        Open,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestScope {
        mode: &'static str,
    }

    fn create_test_leaf() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        KeyChild::leaf(
            TestKey('q'),
            TestAction::Quit,
            "quit application",
            TestCategory::General,
            TestScope { mode: "normal" },
        )
    }

    fn create_test_branch() -> KeyChild<TestKey, TestScope, TestAction, TestCategory> {
        let save_child = KeyChild::leaf(
            TestKey('s'),
            TestAction::Save,
            "save file",
            TestCategory::General,
            TestScope { mode: "normal" },
        );
        let open_child = KeyChild::leaf(
            TestKey('o'),
            TestAction::Open,
            "open file",
            TestCategory::Navigation,
            TestScope { mode: "normal" },
        );
        KeyChild::branch(
            TestKey('f'),
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
            scope: TestScope { mode: "normal" },
        }]);

        let child = KeyChild::new(TestKey('q'), node.clone());

        assert_eq!(child.key, TestKey('q'));
        assert!(matches!(child.node, KeyNode::Leaf(_)));
    }

    #[test]
    fn leaf_entry_stores_all_fields() {
        let entry = LeafEntry {
            action: TestAction::Save,
            description: "save file",
            category: TestCategory::Navigation,
            scope: TestScope { mode: "insert" },
        };

        assert_eq!(entry.action, TestAction::Save);
        assert_eq!(entry.description, "save file");
        assert_eq!(entry.category, TestCategory::Navigation);
        assert_eq!(entry.scope.mode, "insert");
    }

    #[test]
    fn leaf_binding_stores_all_fields() {
        let binding = LeafBinding {
            key: TestKey('w'),
            action: TestAction::Save,
            description: "write file",
            category: TestCategory::General,
            scope: TestScope { mode: "normal" },
        };

        assert_eq!(binding.key, TestKey('w'));
        assert_eq!(binding.action, TestAction::Save);
        assert_eq!(binding.description, "write file");
        assert_eq!(binding.category, TestCategory::General);
        assert_eq!(binding.scope.mode, "normal");
    }

    #[test]
    fn branch_with_empty_children() {
        let child: KeyChild<TestKey, TestScope, TestAction, TestCategory> =
            KeyChild::branch(TestKey('x'), "empty branch", vec![]);

        assert!(child.node.is_branch());
        assert_eq!(child.node.description(), "empty branch");
    }

    #[test]
    fn nested_branch_structure() {
        let inner_leaf = KeyChild::leaf(
            TestKey('d'),
            TestAction::Quit,
            "delete",
            TestCategory::General,
            TestScope { mode: "normal" },
        );
        let inner_branch = KeyChild::branch(TestKey('d'), "delete operations", vec![inner_leaf]);
        let outer_branch = KeyChild::branch(TestKey('g'), "go to", vec![inner_branch]);

        assert!(outer_branch.node.is_branch());

        if let KeyNode::Branch { children, .. } = outer_branch.node {
            assert_eq!(children.len(), 1);
            assert!(children[0].node.is_branch());
        } else {
            panic!("expected branch node");
        }
    }
}
