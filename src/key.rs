use std::hash::Hash;

pub trait Key: Clone + PartialEq + Eq + Send + Sync + 'static {
    fn display(&self) -> String;
    fn is_escape(&self) -> bool;
    fn is_backspace(&self) -> bool;

    fn from_char(_c: char) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn from_special_name(_name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    fn space() -> Self
    where
        Self: Sized;
}

#[cfg(feature = "crossterm")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CrosstermKey {
    Char(char),
    Tab,
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    F(u8),
    Ctrl(char),
}

#[cfg(feature = "crossterm")]
impl CrosstermKey {
    pub fn from_keycode(
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Option<Self> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match code {
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    Some(CrosstermKey::Ctrl(c.to_ascii_lowercase()))
                } else {
                    Some(CrosstermKey::Char(c))
                }
            }
            KeyCode::Tab => Some(CrosstermKey::Tab),
            KeyCode::Enter => Some(CrosstermKey::Enter),
            KeyCode::Backspace => Some(CrosstermKey::Backspace),
            KeyCode::Esc => Some(CrosstermKey::Esc),
            KeyCode::Up => Some(CrosstermKey::Up),
            KeyCode::Down => Some(CrosstermKey::Down),
            KeyCode::Left => Some(CrosstermKey::Left),
            KeyCode::Right => Some(CrosstermKey::Right),
            KeyCode::Home => Some(CrosstermKey::Home),
            KeyCode::End => Some(CrosstermKey::End),
            KeyCode::PageUp => Some(CrosstermKey::PageUp),
            KeyCode::PageDown => Some(CrosstermKey::PageDown),
            KeyCode::F(n) => Some(CrosstermKey::F(n)),
            _ => None,
        }
    }
}

#[cfg(feature = "crossterm")]
impl Key for CrosstermKey {
    fn display(&self) -> String {
        match self {
            CrosstermKey::Char(' ') => "Space".to_string(),
            CrosstermKey::Char(c) => c.to_string(),
            CrosstermKey::Tab => "Tab".to_string(),
            CrosstermKey::Enter => "Enter".to_string(),
            CrosstermKey::Backspace => "Backspace".to_string(),
            CrosstermKey::Esc => "Esc".to_string(),
            CrosstermKey::Up => "↑".to_string(),
            CrosstermKey::Down => "↓".to_string(),
            CrosstermKey::Left => "←".to_string(),
            CrosstermKey::Right => "→".to_string(),
            CrosstermKey::Home => "Home".to_string(),
            CrosstermKey::End => "End".to_string(),
            CrosstermKey::PageUp => "PageUp".to_string(),
            CrosstermKey::PageDown => "PageDown".to_string(),
            CrosstermKey::F(n) => format!("F{}", n),
            CrosstermKey::Ctrl(c) => format!("<C-{}>", c),
        }
    }

    fn is_escape(&self) -> bool {
        matches!(self, CrosstermKey::Esc)
    }

    fn is_backspace(&self) -> bool {
        matches!(self, CrosstermKey::Backspace)
    }

    fn space() -> Self {
        CrosstermKey::Char(' ')
    }

    fn from_char(c: char) -> Option<Self> {
        Some(CrosstermKey::Char(c))
    }

    fn from_special_name(name: &str) -> Option<Self> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "tab" => Some(CrosstermKey::Tab),
            "enter" => Some(CrosstermKey::Enter),
            "bs" | "backspace" => Some(CrosstermKey::Backspace),
            "esc" | "escape" => Some(CrosstermKey::Esc),
            "up" => Some(CrosstermKey::Up),
            "down" => Some(CrosstermKey::Down),
            "left" => Some(CrosstermKey::Left),
            "right" => Some(CrosstermKey::Right),
            "home" => Some(CrosstermKey::Home),
            "end" => Some(CrosstermKey::End),
            "pgup" | "pageup" => Some(CrosstermKey::PageUp),
            "pgdn" | "pagedown" => Some(CrosstermKey::PageDown),
            "leader" | "space" => Some(CrosstermKey::Char(' ')),
            s if s.starts_with('f') && s.len() > 1 => {
                let num: u8 = s[1..].parse().ok()?;
                if num >= 1 && num <= 12 {
                    Some(CrosstermKey::F(num))
                } else {
                    None
                }
            }
            s if s.starts_with("c-") && s.len() == 3 => {
                let c = s.chars().nth(2)?;
                Some(CrosstermKey::Ctrl(c.to_ascii_lowercase()))
            }
            _ => None,
        }
    }
}

#[cfg(feature = "crossterm")]
impl From<crossterm::event::KeyCode> for CrosstermKey {
    fn from(code: crossterm::event::KeyCode) -> Self {
        use crossterm::event::KeyCode;

        match code {
            KeyCode::Char(c) => CrosstermKey::Char(c),
            KeyCode::Tab => CrosstermKey::Tab,
            KeyCode::Enter => CrosstermKey::Enter,
            KeyCode::Backspace => CrosstermKey::Backspace,
            KeyCode::Esc => CrosstermKey::Esc,
            KeyCode::Up => CrosstermKey::Up,
            KeyCode::Down => CrosstermKey::Down,
            KeyCode::Left => CrosstermKey::Left,
            KeyCode::Right => CrosstermKey::Right,
            KeyCode::Home => CrosstermKey::Home,
            KeyCode::End => CrosstermKey::End,
            KeyCode::PageUp => CrosstermKey::PageUp,
            KeyCode::PageDown => CrosstermKey::PageDown,
            KeyCode::F(n) => CrosstermKey::F(n),
            _ => CrosstermKey::Char('\0'),
        }
    }
}

pub fn parse_key_sequence<K: Key>(s: &str) -> Vec<K> {
    let mut keys = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            let mut special = String::new();
            while let Some(&next) = chars.peek() {
                if next == '>' {
                    chars.next();
                    break;
                }
                special.push(chars.next().unwrap());
            }
            if let Some(key) = K::from_special_name(&special) {
                keys.push(key);
            }
        } else if let Some(key) = K::from_char(c) {
            keys.push(key);
        }
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "crossterm")]
    mod crossterm_tests {
        use super::*;

        #[test]
        fn display_returns_space_for_space_char() {
            let key = CrosstermKey::Char(' ');
            assert_eq!(key.display(), "Space");
        }

        #[test]
        fn display_returns_char_for_regular_chars() {
            assert_eq!(CrosstermKey::Char('a').display(), "a");
            assert_eq!(CrosstermKey::Char('Z').display(), "Z");
            assert_eq!(CrosstermKey::Char('1').display(), "1");
        }

        #[test]
        fn display_returns_special_key_names() {
            assert_eq!(CrosstermKey::Tab.display(), "Tab");
            assert_eq!(CrosstermKey::Enter.display(), "Enter");
            assert_eq!(CrosstermKey::Backspace.display(), "Backspace");
            assert_eq!(CrosstermKey::Esc.display(), "Esc");
            assert_eq!(CrosstermKey::Home.display(), "Home");
            assert_eq!(CrosstermKey::End.display(), "End");
            assert_eq!(CrosstermKey::PageUp.display(), "PageUp");
            assert_eq!(CrosstermKey::PageDown.display(), "PageDown");
        }

        #[test]
        fn display_returns_arrows() {
            assert_eq!(CrosstermKey::Up.display(), "↑");
            assert_eq!(CrosstermKey::Down.display(), "↓");
            assert_eq!(CrosstermKey::Left.display(), "←");
            assert_eq!(CrosstermKey::Right.display(), "→");
        }

        #[test]
        fn display_returns_function_keys() {
            assert_eq!(CrosstermKey::F(1).display(), "F1");
            assert_eq!(CrosstermKey::F(12).display(), "F12");
        }

        #[test]
        fn display_returns_ctrl_combinations() {
            assert_eq!(CrosstermKey::Ctrl('x').display(), "<C-x>");
            assert_eq!(CrosstermKey::Ctrl('c').display(), "<C-c>");
        }

        #[test]
        fn is_escape_returns_true_for_esc() {
            assert!(CrosstermKey::Esc.is_escape());
        }

        #[test]
        fn is_escape_returns_false_for_other_keys() {
            assert!(!CrosstermKey::Char('a').is_escape());
            assert!(!CrosstermKey::Enter.is_escape());
            assert!(!CrosstermKey::Tab.is_escape());
        }

        #[test]
        fn is_backspace_returns_true_for_backspace() {
            assert!(CrosstermKey::Backspace.is_backspace());
        }

        #[test]
        fn is_backspace_returns_false_for_other_keys() {
            assert!(!CrosstermKey::Char('a').is_backspace());
            assert!(!CrosstermKey::Enter.is_backspace());
            assert!(!CrosstermKey::Esc.is_backspace());
        }

        #[test]
        fn space_returns_space_char() {
            assert_eq!(CrosstermKey::space(), CrosstermKey::Char(' '));
        }

        #[test]
        fn from_keycode_converts_char() {
            use crossterm::event::{KeyCode, KeyModifiers};

            let key = CrosstermKey::from_keycode(KeyCode::Char('a'), KeyModifiers::empty());
            assert_eq!(key, Some(CrosstermKey::Char('a')));
        }

        #[test]
        fn from_keycode_converts_ctrl_char() {
            use crossterm::event::{KeyCode, KeyModifiers};

            let key = CrosstermKey::from_keycode(KeyCode::Char('x'), KeyModifiers::CONTROL);
            assert_eq!(key, Some(CrosstermKey::Ctrl('x')));
        }

        #[test]
        fn from_keycode_converts_special_keys() {
            use crossterm::event::{KeyCode, KeyModifiers};

            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Tab, KeyModifiers::empty()),
                Some(CrosstermKey::Tab)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Enter, KeyModifiers::empty()),
                Some(CrosstermKey::Enter)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Backspace, KeyModifiers::empty()),
                Some(CrosstermKey::Backspace)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Esc, KeyModifiers::empty()),
                Some(CrosstermKey::Esc)
            );
        }

        #[test]
        fn from_keycode_converts_arrow_keys() {
            use crossterm::event::{KeyCode, KeyModifiers};

            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Up, KeyModifiers::empty()),
                Some(CrosstermKey::Up)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Down, KeyModifiers::empty()),
                Some(CrosstermKey::Down)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Left, KeyModifiers::empty()),
                Some(CrosstermKey::Left)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Right, KeyModifiers::empty()),
                Some(CrosstermKey::Right)
            );
        }

        #[test]
        fn from_keycode_converts_function_keys() {
            use crossterm::event::{KeyCode, KeyModifiers};

            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::F(1), KeyModifiers::empty()),
                Some(CrosstermKey::F(1))
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::F(12), KeyModifiers::empty()),
                Some(CrosstermKey::F(12))
            );
        }

        #[test]
        fn from_keycode_converts_navigation_keys() {
            use crossterm::event::{KeyCode, KeyModifiers};

            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::Home, KeyModifiers::empty()),
                Some(CrosstermKey::Home)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::End, KeyModifiers::empty()),
                Some(CrosstermKey::End)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::PageUp, KeyModifiers::empty()),
                Some(CrosstermKey::PageUp)
            );
            assert_eq!(
                CrosstermKey::from_keycode(KeyCode::PageDown, KeyModifiers::empty()),
                Some(CrosstermKey::PageDown)
            );
        }

        #[test]
        fn from_keycode_ctrl_normalizes_to_lowercase() {
            use crossterm::event::{KeyCode, KeyModifiers};

            let key = CrosstermKey::from_keycode(KeyCode::Char('X'), KeyModifiers::CONTROL);
            assert_eq!(key, Some(CrosstermKey::Ctrl('x')));
        }

        #[test]
        fn from_special_name_parses_tab() {
            assert_eq!(
                CrosstermKey::from_special_name("tab"),
                Some(CrosstermKey::Tab)
            );
            assert_eq!(
                CrosstermKey::from_special_name("TAB"),
                Some(CrosstermKey::Tab)
            );
        }

        #[test]
        fn from_special_name_parses_enter() {
            assert_eq!(
                CrosstermKey::from_special_name("enter"),
                Some(CrosstermKey::Enter)
            );
        }

        #[test]
        fn from_special_name_parses_backspace() {
            assert_eq!(
                CrosstermKey::from_special_name("bs"),
                Some(CrosstermKey::Backspace)
            );
            assert_eq!(
                CrosstermKey::from_special_name("backspace"),
                Some(CrosstermKey::Backspace)
            );
        }

        #[test]
        fn from_special_name_parses_escape() {
            assert_eq!(
                CrosstermKey::from_special_name("esc"),
                Some(CrosstermKey::Esc)
            );
            assert_eq!(
                CrosstermKey::from_special_name("escape"),
                Some(CrosstermKey::Esc)
            );
        }

        #[test]
        fn from_special_name_parses_arrow_keys() {
            assert_eq!(
                CrosstermKey::from_special_name("up"),
                Some(CrosstermKey::Up)
            );
            assert_eq!(
                CrosstermKey::from_special_name("down"),
                Some(CrosstermKey::Down)
            );
            assert_eq!(
                CrosstermKey::from_special_name("left"),
                Some(CrosstermKey::Left)
            );
            assert_eq!(
                CrosstermKey::from_special_name("right"),
                Some(CrosstermKey::Right)
            );
        }

        #[test]
        fn from_special_name_parses_navigation_keys() {
            assert_eq!(
                CrosstermKey::from_special_name("home"),
                Some(CrosstermKey::Home)
            );
            assert_eq!(
                CrosstermKey::from_special_name("end"),
                Some(CrosstermKey::End)
            );
        }

        #[test]
        fn from_special_name_parses_page_keys() {
            assert_eq!(
                CrosstermKey::from_special_name("pgup"),
                Some(CrosstermKey::PageUp)
            );
            assert_eq!(
                CrosstermKey::from_special_name("pageup"),
                Some(CrosstermKey::PageUp)
            );
            assert_eq!(
                CrosstermKey::from_special_name("pgdn"),
                Some(CrosstermKey::PageDown)
            );
            assert_eq!(
                CrosstermKey::from_special_name("pagedown"),
                Some(CrosstermKey::PageDown)
            );
        }

        #[test]
        fn from_special_name_parses_function_keys() {
            assert_eq!(
                CrosstermKey::from_special_name("f1"),
                Some(CrosstermKey::F(1))
            );
            assert_eq!(
                CrosstermKey::from_special_name("f12"),
                Some(CrosstermKey::F(12))
            );
            assert_eq!(CrosstermKey::from_special_name("f0"), None);
            assert_eq!(CrosstermKey::from_special_name("f13"), None);
        }

        #[test]
        fn from_special_name_parses_leader_and_space() {
            assert_eq!(
                CrosstermKey::from_special_name("leader"),
                Some(CrosstermKey::Char(' '))
            );
            assert_eq!(
                CrosstermKey::from_special_name("space"),
                Some(CrosstermKey::Char(' '))
            );
        }

        #[test]
        fn from_special_name_parses_ctrl_keys() {
            assert_eq!(
                CrosstermKey::from_special_name("c-x"),
                Some(CrosstermKey::Ctrl('x'))
            );
            assert_eq!(
                CrosstermKey::from_special_name("C-X"),
                Some(CrosstermKey::Ctrl('x'))
            );
            assert_eq!(
                CrosstermKey::from_special_name("c-a"),
                Some(CrosstermKey::Ctrl('a'))
            );
        }

        #[test]
        fn from_special_name_returns_none_for_unknown() {
            assert_eq!(CrosstermKey::from_special_name("unknown"), None);
            assert_eq!(CrosstermKey::from_special_name("foo"), None);
        }

        #[test]
        fn parse_key_sequence_parses_single_char() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("a");
            assert_eq!(keys, vec![CrosstermKey::Char('a')]);
        }

        #[test]
        fn parse_key_sequence_parses_multiple_chars() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("abc");
            assert_eq!(
                keys,
                vec![
                    CrosstermKey::Char('a'),
                    CrosstermKey::Char('b'),
                    CrosstermKey::Char('c'),
                ]
            );
        }

        #[test]
        fn parse_key_sequence_parses_special_key() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<enter>");
            assert_eq!(keys, vec![CrosstermKey::Enter]);
        }

        #[test]
        fn parse_key_sequence_parses_ctrl_key() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<c-x>");
            assert_eq!(keys, vec![CrosstermKey::Ctrl('x')]);
        }

        #[test]
        fn parse_key_sequence_parses_leader_key() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<leader>");
            assert_eq!(keys, vec![CrosstermKey::Char(' ')]);
        }

        #[test]
        fn parse_key_sequence_parses_mixed_sequence() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<leader>m");
            assert_eq!(keys, vec![CrosstermKey::Char(' '), CrosstermKey::Char('m')]);
        }

        #[test]
        fn parse_key_sequence_parses_complex_sequence() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<c-h><enter>q");
            assert_eq!(
                keys,
                vec![
                    CrosstermKey::Ctrl('h'),
                    CrosstermKey::Enter,
                    CrosstermKey::Char('q'),
                ]
            );
        }

        #[test]
        fn parse_key_sequence_returns_empty_for_empty_string() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("");
            assert!(keys.is_empty());
        }

        #[test]
        fn parse_key_sequence_handles_case_insensitive_special() {
            let keys: Vec<CrosstermKey> = parse_key_sequence("<ENTER>");
            assert_eq!(keys, vec![CrosstermKey::Enter]);

            let keys: Vec<CrosstermKey> = parse_key_sequence("<TAB>");
            assert_eq!(keys, vec![CrosstermKey::Tab]);
        }
    }
}
