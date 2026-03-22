#[derive(Debug, Clone, PartialEq)]
pub struct KeyResult<A> {
    pub action: Option<A>,
}

impl<A> KeyResult<A> {
    pub fn with_action(action: A) -> Self {
        Self {
            action: Some(action),
        }
    }

    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }
}
