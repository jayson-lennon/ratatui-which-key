/// Result of processing a keybinding, containing an optional action to execute.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyResult<A> {
    /// The action to execute, if any.
    pub action: Option<A>,
}

impl<A> KeyResult<A> {
    /// Creates a new `KeyResult` with the given action.
    pub fn with_action(action: A) -> Self {
        Self {
            action: Some(action),
        }
    }

    /// Returns `true` if this result contains an action.
    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }
}
