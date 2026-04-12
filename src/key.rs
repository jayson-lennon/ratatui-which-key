// Copyright (C) 2026 Jayson Lennon
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program; if not, see <https://opensource.org/license/lgpl-3-0>.

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

/// Parses a key sequence string into a vector of keys.
///
/// # Panics
///
/// Panics if special character parsing fails when calling `chars.next().unwrap()`.
/// This can occur if the iterator is empty after peeking a character.
pub fn parse_key_sequence<K: Key>(s: &str, leader: &K) -> Vec<K> {
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
            if special.eq_ignore_ascii_case("leader") {
                keys.push(leader.clone());
                continue;
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
    use crossterm::event::{KeyCode, KeyModifiers};

    use crate::parse_key_sequence;

    fn leader() -> crossterm::event::KeyEvent {
        crossterm::event::KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::empty())
    }

    #[test]
    fn lt_parses_to_literal_less_than() {
        // Given a key sequence containing <lt>.
        let keys = parse_key_sequence("<lt>", &leader());

        // Then it produces a single '<' key.
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].code, KeyCode::Char('<'));
        assert_eq!(keys[0].modifiers, KeyModifiers::empty());
    }

    #[test]
    fn gt_parses_to_literal_greater_than() {
        // Given a key sequence containing <gt>.
        let keys = parse_key_sequence("<gt>", &leader());

        // Then it produces a single '>' key.
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].code, KeyCode::Char('>'));
        assert_eq!(keys[0].modifiers, KeyModifiers::empty());
    }

    #[test]
    fn lt_and_gt_in_mixed_sequence() {
        // Given a key sequence mixing regular chars with <lt> and <gt>.
        let keys = parse_key_sequence("a<lt>b<gt>c", &leader());

        // Then it produces the five expected keys.
        assert_eq!(keys.len(), 5);
        assert_eq!(keys[0].code, KeyCode::Char('a'));
        assert_eq!(keys[1].code, KeyCode::Char('<'));
        assert_eq!(keys[2].code, KeyCode::Char('b'));
        assert_eq!(keys[3].code, KeyCode::Char('>'));
        assert_eq!(keys[4].code, KeyCode::Char('c'));
    }

    #[test]
    fn consecutive_lt_gt_parses_to_angle_bracket_pair() {
        // Given a key sequence of <lt><gt>.
        let keys = parse_key_sequence("<lt><gt>", &leader());

        // Then it produces '<' then '>'.
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].code, KeyCode::Char('<'));
        assert_eq!(keys[1].code, KeyCode::Char('>'));
    }
}
