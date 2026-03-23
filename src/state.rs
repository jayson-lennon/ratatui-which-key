// Copyright (C) 2026 Jayson Lennon
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{BindingGroup, Key, KeyResult, Keymap, NodeResult};

/// State for the which-key widget.
///
/// Holds all runtime data including the keymap, current scope,
/// and pending key sequence.
#[derive(Debug, Clone)]
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
        if key.is_backspace() {
            self.current_sequence.pop();
            if self.current_sequence.is_empty() {
                self.dismiss();
            }
            return KeyResult { action: None };
        }

        self.current_sequence.push(key.clone());
        match self.keymap.navigate(&self.current_sequence, &self.scope) {
            Some(NodeResult::Branch { .. }) => {
                self.active = true;
                KeyResult { action: None }
            }
            Some(NodeResult::Leaf { action }) => {
                self.active = false;
                self.current_sequence.clear();
                KeyResult::with_action(action)
            }
            None => {
                self.dismiss();
                KeyResult { action: None }
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

    impl std::fmt::Display for TestAction {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestAction::Quit => write!(f, "quit"),
                TestAction::Save => write!(f, "save"),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum TestScope {
        Global,
        Insert,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestKey {
        Char(char),
        Backspace,
    }

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
    fn toggle_activates_inactive_state() {
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);
        assert!(!state.active);

        state.toggle();

        assert!(state.active);
    }

    #[test]
    fn toggle_deactivates_active_state() {
        let keymap = create_test_keymap();
        let mut state = WhichKeyState::new(keymap, TestScope::Global);
        state.active = true;

        state.toggle();

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

    use crate::test_utils::state_with_binding_and_sequence;

    #[test]
    fn leaf_action_clears_sequence() {
        let mut state = state_with_binding_and_sequence(
            "qw",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
            &[],
        );

        state.handle_key(TestKey::Char('q'));
        let result = state.handle_key(TestKey::Char('w'));

        assert!(result.has_action());
        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
        assert_eq!(state.format_path(), "");
    }

    #[test]
    fn backspace_dismisses_when_single_key_in_sequence() {
        let mut state = state_with_binding_and_sequence(
            "qw",
            TestAction::Quit,
            TestCategory::General,
            TestScope::Global,
            &[TestKey::Char('q')],
        );

        state.handle_key(TestKey::Backspace);

        assert!(!state.active);
        assert!(state.current_sequence.is_empty());
    }
}
