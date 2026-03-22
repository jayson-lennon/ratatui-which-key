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
        let result: KeyResult<()> = KeyResult::consumed();

        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(!result.dismiss);
    }

    #[test]
    fn ignored_returns_key_result_with_consumed_false() {
        let result: KeyResult<()> = KeyResult::ignored();

        assert!(!result.consumed);
        assert!(result.action.is_none());
        assert!(!result.dismiss);
    }

    #[test]
    fn with_action_returns_key_result_with_action_and_dismiss() {
        let result = KeyResult::with_action("test_action");

        assert!(result.consumed);
        assert_eq!(result.action, Some("test_action"));
        assert!(result.dismiss);
    }

    #[test]
    fn dismiss_returns_key_result_with_dismiss_true() {
        let result: KeyResult<()> = KeyResult::dismiss();

        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(result.dismiss);
    }

    #[test]
    fn and_dismiss_sets_dismiss_to_true() {
        let result: KeyResult<()> = KeyResult::consumed().and_dismiss();

        assert!(result.consumed);
        assert!(result.action.is_none());
        assert!(result.dismiss);
    }
}
