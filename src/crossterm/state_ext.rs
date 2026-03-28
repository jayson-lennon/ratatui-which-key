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

use crate::WhichKeyState;
use crossterm::event::{Event, KeyEvent};

use super::event_result::EventResult;
use super::keymap_ext::CrosstermKeymapExt;

/// Extension trait for WhichKeyState to handle crossterm events.
pub trait CrosstermStateExt<S, A, C> {
    /// Handle a crossterm event.
    fn handle_event(&mut self, event: Event) -> EventResult<A>;
}

impl<S, A, C> CrosstermStateExt<S, A, C> for WhichKeyState<KeyEvent, S, A, C>
where
    S: Clone + Ord + PartialEq + Send + Sync + 'static,
    A: Clone + Send + Sync + 'static,
    C: Clone + std::fmt::Display + 'static,
{
    fn handle_event(&mut self, event: Event) -> EventResult<A> {
        match event {
            Event::Key(key) => {
                let result = self.handle_key(key);
                EventResult::Key(result)
            }
            Event::Mouse(mouse) => {
                let action = self
                    .keymap()
                    .mouse_handler()
                    .and_then(|h| h(mouse, self.scope()));
                EventResult::Mouse(action)
            }
            Event::Resize(cols, rows) => {
                let action = self
                    .keymap()
                    .resize_handler()
                    .and_then(|h| h(cols, rows, self.scope()));
                EventResult::Resize(action)
            }
            Event::FocusGained => {
                let action = self
                    .keymap()
                    .focus_gained_handler()
                    .and_then(|h| h(self.scope()));
                EventResult::FocusGained(action)
            }
            Event::FocusLost => {
                let action = self
                    .keymap()
                    .focus_lost_handler()
                    .and_then(|h| h(self.scope()));
                EventResult::FocusLost(action)
            }
            _ => EventResult::Unhandled,
        }
    }
}
