use crate::{Key, Keymap};

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
    use crate::test_utils::{TestAction, TestCategory, TestScope};
    use crate::{CrosstermKey, KeyNode};

    #[test]
    fn bind_auto_applies_scope_and_category() {
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        let mut builder =
            ScopeAndCategoryBuilder::new(&mut keymap, TestScope::Global, TestCategory::Navigation);

        builder.bind("h", TestAction::Open);

        let node = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);
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
        let mut keymap: Keymap<CrosstermKey, TestScope, TestAction, TestCategory> = Keymap::new();
        let mut builder =
            ScopeAndCategoryBuilder::new(&mut keymap, TestScope::Insert, TestCategory::General);

        builder
            .bind("q", TestAction::Quit)
            .bind("s", TestAction::Save)
            .bind("o", TestAction::Open);

        let quit_node = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);
        let save_node = keymap.get_node_at_path(&[CrosstermKey::Char('s')]);
        let open_node = keymap.get_node_at_path(&[CrosstermKey::Char('o')]);

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
