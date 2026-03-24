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

use crate::KeyResult;

/// Result of handling a crossterm event.
#[derive(Debug, Clone)]
pub enum EventResult<A> {
    /// Key event was processed.
    Key(KeyResult<A>),
    /// Mouse event was processed.
    Mouse(Option<A>),
    /// Resize event was processed.
    Resize(Option<A>),
    /// Focus gained event was processed.
    FocusGained(Option<A>),
    /// Focus lost event was processed.
    FocusLost(Option<A>),
    /// Event was not handled.
    Unhandled,
}

impl<A> EventResult<A> {
    /// Returns a reference to the action, if any.
    pub fn action(&self) -> Option<&A> {
        match self {
            EventResult::Key(kr) => kr.action.as_ref(),
            EventResult::Mouse(a) => a.as_ref(),
            EventResult::Resize(a) => a.as_ref(),
            EventResult::FocusGained(a) => a.as_ref(),
            EventResult::FocusLost(a) => a.as_ref(),
            EventResult::Unhandled => None,
        }
    }

    /// Returns true if an action was produced.
    pub fn has_action(&self) -> bool {
        self.action().is_some()
    }

    /// Converts into the action, if any.
    pub fn into_action(self) -> Option<A> {
        match self {
            EventResult::Key(kr) => kr.action,
            EventResult::Mouse(a) => a,
            EventResult::Resize(a) => a,
            EventResult::FocusGained(a) => a,
            EventResult::FocusLost(a) => a,
            EventResult::Unhandled => None,
        }
    }
}
