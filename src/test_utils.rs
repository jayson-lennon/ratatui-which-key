#[cfg(test)]
use crate::Key;

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestKey {
    Char(char),
    Esc,
    Backspace,
}

#[cfg(test)]
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

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub enum TestAction {
    Quit,
    Save,
    Open,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub enum TestScope {
    Global,
    Insert,
    Normal,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCategory {
    General,
    Navigation,
}
