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

/// Result of processing a keybinding, containing an optional action to execute.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyResult<A> {
    /// The action to execute, if any.
    pub action: Option<A>,
}

impl<A> KeyResult<A> {
    /// Creates a new `KeyResult` with the given action.
    pub fn with_action(action: A) -> Self {
        Self {
            action: Some(action),
        }
    }

    /// Returns `true` if this result contains an action.
    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }
}
