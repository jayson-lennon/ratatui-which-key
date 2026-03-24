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
mod tests {}
