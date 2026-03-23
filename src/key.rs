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

use std::hash::Hash;

/// A trait for abstracting keyboard key representation.
///
/// This trait enables different backends to implement their own key types
/// while providing a common interface for display and conversion.
pub trait Key: Clone + PartialEq + Eq + Send + Sync + 'static {
    /// Returns a human-readable string representation of the key.
    fn display(&self) -> String;
    /// Returns true if this key represents backspace.
    fn is_backspace(&self) -> bool;

    /// Creates a key from a character, if supported.
    fn from_char(_c: char) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    /// Creates a key from a special name (e.g., "enter", "tab", "c-x").
    fn from_special_name(_name: &str) -> Option<Self>
    where
        Self: Sized,
    {
        None
    }

    /// Creates a space key.
    fn space() -> Self
    where
        Self: Sized;
}

/// A key representation for the crossterm backend.
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

impl std::fmt::Display for CrosstermKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrosstermKey::Char(c) => write!(f, "{c}"),
            CrosstermKey::Tab => write!(f, "Tab"),
            CrosstermKey::Enter => write!(f, "Enter"),
            CrosstermKey::Backspace => write!(f, "Backspace"),
            CrosstermKey::Esc => write!(f, "Esc"),
            CrosstermKey::Up => write!(f, "Up"),
            CrosstermKey::Down => write!(f, "Down"),
            CrosstermKey::Left => write!(f, "Left"),
            CrosstermKey::Right => write!(f, "Right"),
            CrosstermKey::Home => write!(f, "Home"),
            CrosstermKey::End => write!(f, "End"),
            CrosstermKey::PageUp => write!(f, "PageUp"),
            CrosstermKey::PageDown => write!(f, "PageDown"),
            CrosstermKey::F(num) => write!(f, "F{num}"),
            CrosstermKey::Ctrl(c) => write!(f, "Ctrl+{c}"),
        }
    }
}

#[cfg(feature = "crossterm")]
impl CrosstermKey {
    /// Creates a key from crossterm's KeyCode and KeyModifiers.
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
            CrosstermKey::F(n) => format!("F{n}"),
            CrosstermKey::Ctrl(c) => format!("<C-{c}>"),
        }
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
                if (1..=12).contains(&num) {
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

/// Parses a key sequence string into a vector of keys.
///
/// # Panics
///
/// Panics if special character parsing fails when calling `chars.next().unwrap()`.
/// This can occur if the iterator is empty after peeking a character.
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
            // Given a space character key.
            let key = CrosstermKey::Char(' ');

            // When displaying the key.
            let display = key.display();

            // Then it returns "Space".
            assert_eq!(display, "Space");
        }

        #[test]
        fn display_returns_char_for_regular_chars() {
            // Given regular character keys.
            // When displaying each character.
            // Then it returns the character itself.
            assert_eq!(CrosstermKey::Char('a').display(), "a");
            assert_eq!(CrosstermKey::Char('Z').display(), "Z");
            assert_eq!(CrosstermKey::Char('1').display(), "1");
        }

        #[test]
        fn display_returns_control_key_names() {
            // Given control key variants.
            // When displaying each control key.
            // Then it returns the expected name.
            assert_eq!(CrosstermKey::Tab.display(), "Tab");
            assert_eq!(CrosstermKey::Enter.display(), "Enter");
            assert_eq!(CrosstermKey::Backspace.display(), "Backspace");
            assert_eq!(CrosstermKey::Esc.display(), "Esc");
        }

        #[test]
        fn display_returns_navigation_key_names() {
            // Given navigation key variants.
            // When displaying each navigation key.
            // Then it returns the expected name.
            assert_eq!(CrosstermKey::Home.display(), "Home");
            assert_eq!(CrosstermKey::End.display(), "End");
            assert_eq!(CrosstermKey::PageUp.display(), "PageUp");
            assert_eq!(CrosstermKey::PageDown.display(), "PageDown");
        }

        #[test]
        fn display_returns_arrows() {
            // Given arrow key variants.
            // When displaying each arrow key.
            // Then it returns the expected arrow symbol.
            assert_eq!(CrosstermKey::Up.display(), "↑");
            assert_eq!(CrosstermKey::Down.display(), "↓");
            assert_eq!(CrosstermKey::Left.display(), "←");
            assert_eq!(CrosstermKey::Right.display(), "→");
        }

        #[test]
        fn display_returns_function_keys() {
            // Given function key variants.
            // When displaying each function key.
            // Then it returns the expected F key name.
            assert_eq!(CrosstermKey::F(1).display(), "F1");
            assert_eq!(CrosstermKey::F(12).display(), "F12");
        }

        #[test]
        fn display_returns_ctrl_combinations() {
            // Given ctrl key combinations.
            // When displaying each ctrl combination.
            // Then it returns the expected format.
            assert_eq!(CrosstermKey::Ctrl('x').display(), "<C-x>");
            assert_eq!(CrosstermKey::Ctrl('c').display(), "<C-c>");
        }

        #[test]
        fn space_returns_space_char() {
            // Given the space static method is called.
            let key = CrosstermKey::space();

            // Then it returns a space character key.
            assert_eq!(key, CrosstermKey::Char(' '));
        }

        #[test]
        fn from_keycode_converts_char() {
            // Given a character key code without modifiers.
            use crossterm::event::{KeyCode, KeyModifiers};
            let code = KeyCode::Char('a');
            let modifiers = KeyModifiers::empty();

            // When converting from keycode.
            let key = CrosstermKey::from_keycode(code, modifiers);

            // Then it returns the expected character key.
            assert_eq!(key, Some(CrosstermKey::Char('a')));
        }

        #[test]
        fn from_keycode_converts_ctrl_char() {
            // Given a character key code with control modifier.
            use crossterm::event::{KeyCode, KeyModifiers};
            let code = KeyCode::Char('x');
            let modifiers = KeyModifiers::CONTROL;

            // When converting from keycode.
            let key = CrosstermKey::from_keycode(code, modifiers);

            // Then it returns the expected ctrl key.
            assert_eq!(key, Some(CrosstermKey::Ctrl('x')));
        }

        #[test]
        fn from_keycode_converts_special_keys() {
            // Given special key codes.
            use crossterm::event::{KeyCode, KeyModifiers};

            // When converting from keycode.
            // Then each returns the expected special key variant.
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
            // Given arrow key codes.
            use crossterm::event::{KeyCode, KeyModifiers};

            // When converting from keycode.
            // Then each returns the expected arrow key variant.
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
            // Given function key codes.
            use crossterm::event::{KeyCode, KeyModifiers};

            // When converting from keycode.
            // Then each returns the expected function key variant.
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
            // Given navigation key codes.
            use crossterm::event::{KeyCode, KeyModifiers};

            // When converting from keycode.
            // Then each returns the expected navigation key variant.
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
            // Given an uppercase character with control modifier.
            use crossterm::event::{KeyCode, KeyModifiers};
            let code = KeyCode::Char('X');
            let modifiers = KeyModifiers::CONTROL;

            // When converting from keycode.
            let key = CrosstermKey::from_keycode(code, modifiers);

            // Then it returns the lowercase ctrl key.
            assert_eq!(key, Some(CrosstermKey::Ctrl('x')));
        }

        #[test]
        fn from_special_name_parses_tab() {
            // Given tab as the special name.
            // When parsing from special name.
            // Then it returns the Tab variant (case insensitive).
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
            // Given "enter" as the special name.
            // When parsing from special name.
            // Then it returns the Enter variant.
            assert_eq!(
                CrosstermKey::from_special_name("enter"),
                Some(CrosstermKey::Enter)
            );
        }

        #[test]
        fn from_special_name_parses_backspace() {
            // Given "bs" and "backspace" as special names.
            // When parsing from special name.
            // Then each returns the Backspace variant.
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
            // Given "esc" and "escape" as special names.
            // When parsing from special name.
            // Then each returns the Esc variant.
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
            // Given arrow key names as special names.
            // When parsing from special name.
            // Then each returns the expected arrow key variant.
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
            // Given "home" and "end" as special names.
            // When parsing from special name.
            // Then each returns the expected navigation key variant.
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
            // Given page key names as special names.
            // When parsing from special name.
            // Then each returns the expected page key variant.
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
        fn from_special_name_parses_valid_function_keys() {
            // Given valid function key names.
            // When parsing from special name.
            // Then it returns the expected function key variants.
            assert_eq!(
                CrosstermKey::from_special_name("f1"),
                Some(CrosstermKey::F(1))
            );
            assert_eq!(
                CrosstermKey::from_special_name("f12"),
                Some(CrosstermKey::F(12))
            );
        }

        #[test]
        fn from_special_name_returns_none_for_invalid_function_keys() {
            // Given invalid function key names.
            // When parsing from special name.
            // Then it returns None.
            assert_eq!(CrosstermKey::from_special_name("f0"), None);
            assert_eq!(CrosstermKey::from_special_name("f13"), None);
        }

        #[test]
        fn from_special_name_parses_leader_and_space() {
            // Given "leader" and "space" as special names.
            // When parsing from special name.
            // Then each returns a space character key.
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
            // Given ctrl key names as special names.
            // When parsing from special name.
            // Then each returns the expected ctrl key (case insensitive).
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
            // Given unknown special names.
            // When parsing from special name.
            // Then it returns None.
            assert_eq!(CrosstermKey::from_special_name("unknown"), None);
            assert_eq!(CrosstermKey::from_special_name("foo"), None);
        }

        #[test]
        fn parse_key_sequence_parses_single_char() {
            // Given a single character string.
            let input = "a";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns a vector with one character key.
            assert_eq!(keys, vec![CrosstermKey::Char('a')]);
        }

        #[test]
        fn parse_key_sequence_parses_multiple_chars() {
            // Given a string with multiple characters.
            let input = "abc";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns a vector with character keys for each.
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
            // Given a special key sequence string.
            let input = "<enter>";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns a vector with the Enter key.
            assert_eq!(keys, vec![CrosstermKey::Enter]);
        }

        #[test]
        fn parse_key_sequence_parses_mixed_sequence() {
            // Given a mixed sequence string with special key and char.
            let input = "<leader>m";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns a vector with space and character keys.
            assert_eq!(keys, vec![CrosstermKey::Char(' '), CrosstermKey::Char('m')]);
        }

        #[test]
        fn parse_key_sequence_returns_empty_for_empty_string() {
            // Given an empty string.
            let input = "";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns an empty vector.
            assert!(keys.is_empty());
        }

        #[test]
        fn parse_key_sequence_parses_uppercase_enter() {
            // Given an uppercase ENTER sequence.
            let input = "<ENTER>";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns the Enter key.
            assert_eq!(keys, vec![CrosstermKey::Enter]);
        }

        #[test]
        fn parse_key_sequence_parses_uppercase_tab() {
            // Given an uppercase TAB sequence.
            let input = "<TAB>";

            // When parsing the key sequence.
            let keys: Vec<CrosstermKey> = parse_key_sequence(input);

            // Then it returns the Tab key.
            assert_eq!(keys, vec![CrosstermKey::Tab]);
        }
    }
}
