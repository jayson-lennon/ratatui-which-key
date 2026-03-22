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
        let description = child.node.description();

        // Then it is a branch with the correct description.
        assert!(is_branch);
        assert_eq!(description, "empty branch");
    }

    #[test]
    fn nested_branch_structure() {
        // Given a nested branch structure with an inner leaf, inner branch, and outer branch.
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
