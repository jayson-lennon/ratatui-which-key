use crate::{Key, Keymap};

/// Builder for creating keybindings within a specific scope.
///
/// Provides convenience methods that default the scope, reducing
/// boilerplate when defining groups of related keybindings.
pub struct ScopeBuilder<'a, K: Key, S, A, C> {
    keymap: &'a mut Keymap<K, S, A, C>,
    scope: S,
}

impl<'a, K: Key, S, A, C> ScopeBuilder<'a, K, S, A, C> {
    pub(super) fn new(keymap: &'a mut Keymap<K, S, A, C>, scope: S) -> Self {
        Self { keymap, scope }
    }

    /// Adds a keybinding with explicit category.
    pub fn bind(
        &mut self,
        sequence: &str,
        action: A,
        description: &'static str,
        category: C,
    ) -> &mut Self
    where
        K: Clone,
        S: Clone + PartialEq,
        A: Clone,
        C: Clone,
    {
        self.keymap
            .bind(sequence, action, description, category, self.scope.clone());
        self
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::{CrosstermKey, KeyNode};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestCategory {
        General,
        Navigation,
    }

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
    }

    #[test]
    fn bind_with_general_category_works_correctly() {
        // Given a keymap with a scope binding using bind() with General category.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("q", TestAction::Quit, "quit", TestCategory::General);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);

        // Then the binding exists with General category.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].category, TestCategory::General);
            assert_eq!(entries[0].action, TestAction::Quit);
        } else {
            panic!("Expected leaf node with General category");
        }
    }

    #[test]
    fn bind_with_navigation_category_works_correctly() {
        // Given a keymap with a scope binding using bind() with Navigation category.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("h", TestAction::Open, "open", TestCategory::Navigation);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);

        // Then the binding exists with Navigation category.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].category, TestCategory::Navigation);
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("Expected leaf node with Navigation category");
        }
    }

    #[test]
    fn bind_with_explicit_category_works_correctly() {
        // Given a keymap with a scope binding using bind() with explicit category.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder.bind("x", TestAction::Quit, "exit", TestCategory::General);

        // When looking up the binding.
        let node = keymap.get_node_at_path(&[CrosstermKey::Char('x')]);

        // Then the binding exists with the specified category.
        assert!(node.is_some());
        if let Some(KeyNode::Leaf(entries)) = node {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].category, TestCategory::General);
            assert_eq!(entries[0].action, TestAction::Quit);
            assert_eq!(entries[0].description, "exit");
        } else {
            panic!("Expected leaf node with explicit category");
        }
    }

    #[test]
    fn chaining_multiple_bindings_works() {
        // Given a keymap with multiple chained bindings in the same scope.
        let mut keymap = Keymap::new();
        let mut builder = ScopeBuilder::new(&mut keymap, TestScope::Global);
        builder
            .bind("q", TestAction::Quit, "quit", TestCategory::General)
            .bind("h", TestAction::Open, "help", TestCategory::Navigation)
            .bind("s", TestAction::Save, "save", TestCategory::General);

        // When looking up each binding.
        let node_q = keymap.get_node_at_path(&[CrosstermKey::Char('q')]);
        let node_h = keymap.get_node_at_path(&[CrosstermKey::Char('h')]);
        let node_s = keymap.get_node_at_path(&[CrosstermKey::Char('s')]);

        // Then all bindings are registered.
        assert!(node_q.is_some());
        assert!(node_h.is_some());
        assert!(node_s.is_some());

        if let Some(KeyNode::Leaf(entries)) = node_q {
            assert_eq!(entries[0].action, TestAction::Quit);
        } else {
            panic!("Expected leaf node for 'q'");
        }

        if let Some(KeyNode::Leaf(entries)) = node_h {
            assert_eq!(entries[0].action, TestAction::Open);
        } else {
            panic!("Expected leaf node for 'h'");
        }

        if let Some(KeyNode::Leaf(entries)) = node_s {
            assert_eq!(entries[0].action, TestAction::Save);
        } else {
            panic!("Expected leaf node for 's'");
        }
    }
}
