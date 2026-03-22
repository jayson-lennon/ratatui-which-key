//! A which-key popup widget for ratatui applications.
//!
//! This crate provides a popup widget that displays available keybindings,
//! similar to Emacs' which-key or Vim's which-key plugins.
//!
//! # Features
//!
//! - `crossterm` (default): Provides `CrosstermKey` implementation
//!
//! # Example
//!
//! ```ignore
//! use ratatui_which_key::{Keymap, WhichKey, WhichKeyState, CrosstermKey};
//!
//! // Define your action type
//! #[derive(Clone)]
//! enum Action { Quit, Save }
//!
//! // Define your scope type
//! #[derive(Clone, PartialEq)]
//! enum Scope { Global, Insert }
//!
//! // Build the keymap
//! let mut keymap: Keymap<CrosstermKey, Scope, Action> = Keymap::new();
//! keymap.bind("q", Action::Quit, Scope::Global);
//!
//! // Create state
//! let mut state = WhichKeyState::new(keymap, Scope::Global);
//!
//! // In your event loop, handle keys
//! let result = state.handle_key(key);
//! if let Some(action) = result.action {
//!     // dispatch action
//! }
//!
//! // When rendering
//! let widget = WhichKey::new();
//! widget.render(buffer, &mut state);
//! ```

mod category_builder;
mod group_builder;
mod key;
mod keymap;
mod node;
mod render;
mod result;
mod scope_and_category_builder;
mod scope_builder;
mod state;
mod test_utils;
mod types;
mod widget;

pub use category_builder::CategoryBuilder;
pub use group_builder::GroupBuilder;
pub use key::Key;
pub use key::parse_key_sequence;
pub use keymap::Keymap;
pub use node::{KeyChild, KeyNode, LeafBinding, LeafEntry};
pub use result::KeyResult;
pub use scope_and_category_builder::ScopeAndCategoryBuilder;
pub use scope_builder::ScopeBuilder;
pub use state::WhichKeyState;
pub use types::{Binding, BindingGroup, DisplayBinding, NodeResult};
pub use widget::{PopupPosition, WhichKey};

#[cfg(feature = "crossterm")]
pub use key::CrosstermKey;
