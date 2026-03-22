use derive_more::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyResult<A> {
    pub consumed: bool,
    pub action: Option<A>,
    pub dismiss: bool,
}

impl<A> KeyResult<A> {
    pub fn consumed() -> Self {
        Self {
            consumed: true,
            action: None,
            dismiss: false,
        }
    }

    pub fn ignored() -> Self {
        Self {
            consumed: false,
            action: None,
            dismiss: false,
        }
    }

    pub fn with_action(action: A) -> Self {
        Self {
            consumed: true,
            action: Some(action),
            dismiss: true,
        }
    }

    pub fn dismiss() -> Self {
        Self {
            consumed: true,
            action: None,
            dismiss: true,
        }
    }

    #[must_use]
    pub fn and_dismiss(self) -> Self {
        Self {
            dismiss: true,
            ..self
        }
    }

    pub fn is_consumed(&self) -> bool {
        self.consumed
    }

    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumed_returns_key_result_with_consumed_true() {
        // Given a consumed KeyResult.
        let result: KeyResult<()> = KeyResult::consumed();

        // When checking its properties.
        // Then the consumed flag is true, action is none, and dismiss is false.
        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(!result.dismiss);
    }

    #[test]
    fn ignored_returns_key_result_with_consumed_false() {
        // Given an ignored KeyResult.
        let result: KeyResult<()> = KeyResult::ignored();

        // When checking its properties.
        // Then the consumed flag is false, action is none, and dismiss is false.
        assert!(!result.consumed);
        assert!(result.action.is_none());
        assert!(!result.dismiss);
    }

    #[test]
    fn with_action_returns_key_result_with_action_and_dismiss() {
        // Given a KeyResult with an action.
        let result = KeyResult::with_action("test_action");

        // When checking its properties.
        // Then consumed is true, action is Some("test_action"), and dismiss is true.
        assert!(result.consumed);
        assert_eq!(result.action, Some("test_action"));
        assert!(result.dismiss);
    }

    #[test]
    fn dismiss_returns_key_result_with_dismiss_true() {
        // Given a dismissed KeyResult.
        let result: KeyResult<()> = KeyResult::dismiss();

        // When checking its properties.
        // Then consumed is true, action is none, and dismiss is true.
        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(result.dismiss);
    }

    #[test]
    fn and_dismiss_sets_dismiss_to_true() {
        // Given a consumed KeyResult.
        let result: KeyResult<()> = KeyResult::consumed();

        // When calling and_dismiss on it.
        let result = result.and_dismiss();

        // Then dismiss is true while consumed remains true and action is none.
        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(result.dismiss);
    }
}
