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

use crate::Key;

/// A key binding with an associated description.
#[derive(Debug, Clone)]
pub struct Binding<K: Key> {
    /// The key that triggers this binding.
    pub key: K,
    /// A human-readable description of what this binding does.
    pub description: String,
}

/// A group of related key bindings under a shared category.
#[derive(Debug, Clone)]
pub struct BindingGroup<K: Key> {
    /// The category name for this group of bindings.
    pub category: String,
    /// The bindings belonging to this category.
    pub bindings: Vec<Binding<K>>,
}

/// A key binding prepared for display, including category information.
#[derive(Debug, Clone)]
pub struct DisplayBinding<K, C> {
    /// The key that triggers this binding.
    pub key: K,
    /// A human-readable description of what this binding does.
    pub description: String,
    /// The category this binding belongs to.
    pub category: C,
}

/// The result of traversing a node in the key binding tree.
#[derive(Debug, Clone)]
pub enum NodeResult<K: Key, A> {
    /// A branch node with child bindings available for further navigation.
    Branch {
        /// The available child bindings from this branch.
        children: Vec<Binding<K>>,
    },
    /// A leaf node containing the action to execute.
    Leaf {
        /// The action associated with this key sequence.
        action: A,
    },
}
