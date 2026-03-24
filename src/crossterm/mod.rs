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

//! Crossterm backend support for ratatui-which-key.
//!
//! This module provides:
//! - `Key` trait implementation for `crossterm::event::KeyEvent`
//! - Event handlers for mouse, resize, and focus events
//! - `handle_event` method for routing all crossterm events

mod event_result;
mod handlers;
mod key;
mod keymap_ext;
mod state_ext;

pub use event_result::EventResult;
pub use keymap_ext::CrosstermKeymapExt;
pub use state_ext::CrosstermStateExt;
