use crate::{BindingGroup, Key, KeyResult, Keymap, NodeResult};

/// State for the which-key widget.
///
/// Holds all runtime data including the keymap, current scope,
/// and pending key sequence.
pub struct WhichKeyState<K, S, A, C>
where
    K: Key,
{
    /// Whether the popup is visible.
    pub active: bool,
    /// Keys pressed in the current sequence.
    pub current_sequence: Vec<K>,
    /// Current scope for binding resolution.
    scope: S,
    /// The keymap.
    keymap: Keymap<K, S, A, C>,
    /// Cached bindings for the current scope.
    cached_bindings: Vec<BindingGroup<K>>,
}

impl<K, S, A, C> WhichKeyState<K, S, A, C>
where
    K: Key,
    S: Clone,
{
    /// Get the current scope.
    #[must_use]
    pub fn scope(&self) -> &S {
        &self.scope
    }

    /// Toggle popup visibility.
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if self.active {
            self.current_sequence.clear();
        }
    }

    /// Dismiss the popup and clear current sequence.
    pub fn dismiss(&mut self) {
        self.active = false;
        self.current_sequence.clear();
    }

    /// Check if there are pending keys in the sequence.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        !self.current_sequence.is_empty()
    }

    /// Get the keymap reference.
    #[must_use]
    pub fn keymap(&self) -> &Keymap<K, S, A, C> {
        &self.keymap
    }
}

impl<K, S, A, C> WhichKeyState<K, S, A, C>
where
    K: Key + Clone + PartialEq,
    S: Clone + PartialEq + Send + Sync,
    A: Clone + Send + Sync,
    C: Clone + std::fmt::Debug,
{
    /// Create new state with a keymap and initial scope.
    #[must_use]
    pub fn new(keymap: Keymap<K, S, A, C>, scope: S) -> Self {
        let cached_bindings = keymap.bindings_for_scope(scope.clone());
        Self {
            active: false,
            current_sequence: Vec::new(),
            scope,
            keymap,
            cached_bindings,
        }
    }

    /// Update the current scope.
    pub fn set_scope(&mut self, scope: S) {
        self.scope = scope.clone();
        self.cached_bindings = self.keymap.bindings_for_scope(scope);
    }

    /// Handle a key event.
    ///
    /// Returns a `KeyResult` indicating whether the key was consumed,
    /// an action should be dispatched, or the popup should be dismissed.
    pub fn handle_key(&mut self, key: K) -> KeyResult<A> {
        if key.is_escape() {
            self.dismiss();
            return KeyResult::dismiss();
        }

        if key.is_backspace() {
            self.current_sequence.pop();
            if self.current_sequence.is_empty() {
                self.dismiss();
            }
            return KeyResult::consumed();
        }

        self.current_sequence.push(key.clone());
        match self.keymap.navigate(&self.current_sequence) {
            Some(NodeResult::Branch { .. }) => {
                self.active = true;
                KeyResult::consumed()
            }
            Some(NodeResult::Leaf { action }) => {
                self.active = false;
                self.current_sequence.clear();
                KeyResult::with_action(action)
            }
            None => {
                self.dismiss();
                KeyResult::dismiss()
            }
        }
    }

    /// Get bindings for the current state.
    ///
    /// Returns either bindings for the current scope (main view)
    /// or children at the pending path (sequence view).
    #[must_use]
    pub fn current_bindings(&self) -> Vec<BindingGroup<K>> {
        if self.current_sequence.is_empty() {
            self.cached_bindings.clone()
        } else {
            self.keymap
                .children_at_path(&self.current_sequence)
                .map(|children| {
                    vec![BindingGroup {
                        category: String::new(),
                        bindings: children,
                    }]
                })
                .unwrap_or_default()
        }
    }

    /// Format the current sequence as a path string for display.
    #[must_use]
    pub fn format_path(&self) -> String {
        self.current_sequence
            .iter()
            .map(super::key::Key::display)
            .collect::<Vec<_>>()
            .join(" > ")
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestCategory {
        General,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestAction {
        Quit,
        Save,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestScope {
        Global,
        Insert,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestKey {
        Char(char),
        Esc,
        Backspace,
    }

    impl Key for TestKey {
        fn display(&self) -> String {
            match self {
                TestKey::Char(c) => c.to_string(),
                TestKey::Esc => "Esc".to_string(),
                TestKey::Backspace => "BS".to_string(),
            }
        }

        fn is_escape(&self) -> bool {
            matches!(self, TestKey::Esc)
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

    fn create_test_keymap() -> Keymap<TestKey, TestScope, TestAction, TestCategory> {
        Keymap::new()
    }

    #[test]
    fn new_creates_inactive_state() {
        // Given a keymap.
        let keymap = create_test_keymap();

        // When creating a new which-key state.
        let state = WhichKeyState::new(keymap, TestScope::Global);

        // Then the state is inactive with an empty sequence.
        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
    }

    #[test]
    fn toggle_flips_active() {
        // Given a which-key state.
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);

        // When toggling the state.
        state.toggle();

        // Then the state is active.
        assert!(state.active);

        // When toggling again.
        state.toggle();

        // Then the state is inactive.
        assert!(!state.active);
    }

    #[test]
    fn dismiss_clears_state() {
        // Given an active state with a key in the sequence.
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);
        state.active = true;
        state.current_sequence.push(TestKey::Char('a'));

        // When dismissing the state.
        state.dismiss();

        // Then the state is inactive and the sequence is empty.
        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
    }

    #[test]
    fn is_pending_returns_true_when_keys_present() {
        // Given a state with a key in the sequence.
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);
        state.current_sequence.push(TestKey::Char('a'));

        // When checking if pending.
        assert!(state.is_pending());
    }

    #[test]
    fn format_path_joins_keys() {
        // Given a state with multiple keys in the sequence.
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);
        state.current_sequence.push(TestKey::Char('a'));
        state.current_sequence.push(TestKey::Char('b'));

        // When formatting the path.
        assert_eq!(state.format_path(), "a > b");
    }

    #[test]
    fn set_scope_updates_scope() {
        // Given a state with global scope.
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);

        // When setting the scope to insert.
        state.set_scope(TestScope::Insert);

        // Then the scope is updated.
        assert_eq!(*state.scope(), TestScope::Insert);
    }

    #[test]
    fn leaf_action_clears_sequence() {
        // Given a keymap with a leaf action bound to "qw".
        let mut keymap = create_test_keymap();
        keymap.bind(
            "qw",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );
        let mut state = WhichKeyState::new(keymap, TestScope::Global);

        // When pressing 'q' followed by 'w'.
        state.handle_key(TestKey::Char('q'));
        let result = state.handle_key(TestKey::Char('w'));

        // Then the action is triggered and the state is cleared.
        assert!(result.has_action());
        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
        assert_eq!(state.format_path(), "");
    }

    #[test]
    fn backspace_dismisses_when_single_key_in_sequence() {
        // Given a keymap with a bound action and an active state after pressing 'q'.
        let mut keymap = create_test_keymap();
        keymap.bind(
            "qw",
            TestAction::Quit,
            "quit",
            TestCategory::General,
            TestScope::Global,
        );
        let mut state = WhichKeyState::new(keymap, TestScope::Global);

        state.handle_key(TestKey::Char('q'));
        assert!(state.active);
        assert!(!state.current_sequence.is_empty());

        // When pressing backspace.
        state.handle_key(TestKey::Backspace);

        // Then the state is dismissed and the sequence is cleared.
        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
    }
}
