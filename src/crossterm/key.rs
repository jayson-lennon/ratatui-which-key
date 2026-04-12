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

use crate::Key;

impl Key for crossterm::event::KeyEvent {
    fn display(&self) -> String {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Handle Ctrl-modified keys
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char(c) = self.code {
                return format!("<C-{}>", c.to_ascii_lowercase());
            }
        }

        match self.code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::F(n) => format!("F{n}"),
            KeyCode::Null => "Null".to_string(),
            KeyCode::BackTab => "BackTab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::CapsLock => "CapsLock".to_string(),
            KeyCode::ScrollLock => "ScrollLock".to_string(),
            KeyCode::NumLock => "NumLock".to_string(),
            KeyCode::PrintScreen => "PrintScreen".to_string(),
            KeyCode::Pause => "Pause".to_string(),
            KeyCode::Menu => "Menu".to_string(),
            KeyCode::Media(_) => "Media".to_string(),
            KeyCode::KeypadBegin => "KeypadBegin".to_string(),
            KeyCode::Modifier(_) => "Modifier".to_string(),
        }
    }

    fn is_backspace(&self) -> bool {
        matches!(self.code, crossterm::event::KeyCode::Backspace)
    }

    fn space() -> Self {
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(' '),
            crossterm::event::KeyModifiers::empty(),
        )
    }

    fn from_char(c: char) -> Option<Self> {
        Some(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(c),
            crossterm::event::KeyModifiers::empty(),
        ))
    }

    fn from_special_name(name: &str) -> Option<Self> {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let lower = name.to_ascii_lowercase();

        // Handle Ctrl keys: "c-x"
        if lower.starts_with("c-") && lower.len() == 3 {
            let c = lower.chars().nth(2)?;
            return Some(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
        }

        let code = match lower.as_str() {
            "tab" => KeyCode::Tab,
            "enter" => KeyCode::Enter,
            "bs" | "backspace" => KeyCode::Backspace,
            "esc" | "escape" => KeyCode::Esc,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pgup" | "pageup" => KeyCode::PageUp,
            "pgdn" | "pagedown" => KeyCode::PageDown,
            "space" => KeyCode::Char(' '),
            "lt" => KeyCode::Char('<'),
            "gt" => KeyCode::Char('>'),
            s if s.starts_with('f') && s.len() > 1 => {
                let num: u8 = s[1..].parse().ok()?;
                if !(1..=12).contains(&num) {
                    return None;
                }
                KeyCode::F(num)
            }
            _ => return None,
        };

        Some(KeyEvent::new(code, KeyModifiers::empty()))
    }
}
