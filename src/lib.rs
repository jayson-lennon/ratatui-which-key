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

//! A which-key popup widget for ratatui applications.
//!
//! This crate provides a popup widget that displays available keybindings,
//! similar to Neovim's which-key plugin.
//!
//! ## How It Works
//!
//! `ratatui-which-key` requires three data types be defined in your application.
//!
//! ### Scopes
//!
//! The _scope_ is what part of your application is currently "in focus":
//!
//! ```
//! # use ratatui_which_key::{Keymap, WhichKey, WhichKeyState, CrosstermKey};
//! # #[derive(Debug, Clone)]
//! # enum Action {
//! #    Quit,
//! #    Save,
//! #    ToggleHelp,
//! #    MoveDown,
//! #    MoveUp,
//! #    OpenFile,
//! #    SearchFiles,
//! #    SearchBuffers,
//! # }
//! #
//! # impl std::fmt::Display for Action {
//! #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! #         match self {
//! #             Action::Quit => write!(f, "quit"),
//! #             Action::Save => write!(f, "save"),
//! #             Action::ToggleHelp => write!(f, "ToggleHelp"),
//! #             Action::MoveUp => write!(f, "MoveUp"),
//! #             Action::MoveDown => write!(f, "MoveDown"),
//! #             Action::OpenFile => write!(f, "OpenFile"),
//! #             Action::SearchFiles => write!(f, "SearchFiles"),
//! #             Action::SearchBuffers => write!(f, "SearchBuffers"),
//! #         }
//! #     }
//! # }
//! # #[derive(Debug, Clone, PartialEq)]
//! # enum Category { General, Navigation, SearchPanel }
//! # struct App {
//! #     which_key: WhichKeyState<CrosstermKey, Scope, Action, Category>,
//! # }
//! # let mut app = App { which_key: WhichKeyState::new(Keymap::default(), Scope::Global) };
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
//! enum Scope {
//!     Global,
//!     TextInputBox,
//!     SearchPanel,
//!     // ....
//! }
//!
//! // When changing focus to another pane/window/etc:
//! app.which_key.set_scope(Scope::TextInputBox)
//! ```
//!
//! ### Actions
//!
//! `ratatui-which-key` returns an `Action` when a keybind is triggered:
//!
//! ```
//! # use ratatui_which_key::{Keymap, WhichKey, WhichKeyState, CrosstermKey};
//! # #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! # enum Scope { Global, Insert, SearchPanel }
//! # #[derive(Debug, Clone, PartialEq)]
//! # enum Category { General, Navigation, SearchPanel }
//! # struct App {
//! #     which_key: WhichKeyState<CrosstermKey, Scope, Action, Category>,
//! # }
//! # let mut app = App { which_key: WhichKeyState::new(Keymap::default(), Scope::Global) };
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Action {
//!     Quit,
//!     ToggleHelp,
//!     MoveUp,
//!     MoveDown,
//!     Save,
//!     OpenFile,
//!     SearchFiles,
//!     SearchBuffers,
//!     // ...
//! }
//!
//! // Must implement Display to show descriptions in the which-key popup.
//! impl std::fmt::Display for Action {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             Action::Quit => write!(f, "quit"),
//!             Action::ToggleHelp => write!(f, "toggle help"),
//!             Action::MoveUp => write!(f, ""),
//!             Action::MoveDown => write!(f, "move down"),
//!             Action::Save => write!(f, "save"),
//!             Action::OpenFile => write!(f, "open file"),
//!             Action::SearchFiles => write!(f, "search files"),
//!             Action::SearchBuffers => write!(f, "search buffers"),
//!         }
//!     }
//! }
//!
//! // In your input handler:
//! # let key = CrosstermKey::Char('q');
//! if let Some(action) = app.which_key.handle_key(key).action {
//!     match action {
//!         Action::ToggleHelp => app.which_key.toggle(),
//!         Action::Quit => (), // logic here
//!         Action::MoveUp => (),
//!         Action::MoveDown => (),
//!         Action::Save => (),
//!         Action::OpenFile => (),
//!         Action::SearchFiles => (),
//!         Action::SearchBuffers => (),
//!     }
//! }
//! ```
//!
//! ### Categories
//!
//! The `ratatui-which-key` popup displays keybinds sorted by category:
//!
//! ```
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Category {
//!     General,
//!     Navigation,
//!     Search,
//!     // ...
//! }
//! ```
//!
//! ## Keymap Configuration
//!
//! You'll need to put a `WhichKeyState<CrosstermKey, Scope, Action, Category>` at the top-level of your application (like in `App`). Then at program start, configure your keybinds by creating a new `Keymap`. The code comments explain the different ways of performing keybindings.
//!
//! ```
//! # use ratatui_which_key::{Keymap, WhichKey, WhichKeyState, CrosstermKey};
//! # // Define your action type
//! # #[derive(Debug, Clone)]
//! # enum Action {
//! #    Quit,
//! #    Save,
//! #    ToggleHelp,
//! #    MoveDown,
//! #    MoveUp,
//! #    OpenFile,
//! #    SearchFiles,
//! #    SearchBuffers,
//! #    SearchGrep,
//! #    InsertModePrintableChar(char),
//! #    ToNormalMode
//! # }
//! #
//! # impl std::fmt::Display for Action {
//! #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! #         match self {
//! #             Action::Quit => write!(f, "quit"),
//! #             Action::Save => write!(f, "save"),
//! #             Action::ToggleHelp => write!(f, "ToggleHelp"),
//! #             Action::MoveUp => write!(f, "MoveUp"),
//! #             Action::MoveDown => write!(f, "MoveDown"),
//! #             Action::OpenFile => write!(f, "OpenFile"),
//! #             Action::SearchFiles => write!(f, "SearchFiles"),
//! #             Action::SearchBuffers => write!(f, "SearchBuffers"),
//! #             Action::SearchGrep => write!(f, "SearchGrep"),
//! #             Action::InsertModePrintableChar(k) => write!(f, "key: {k}"),
//! #             Action::ToNormalMode => write!(f, "normal mode"),
//! #         }
//! #     }
//! # }
//! # #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! # enum Scope { Global, Insert, SearchPanel }
//! # #[derive(Debug, Clone, PartialEq)]
//! # enum Category { General, Navigation, SearchPanel }
//! struct App {
//!     which_key: WhichKeyState<CrosstermKey, Scope, Action, Category>,
//! }
//!
//! let mut keymap = Keymap::new();
//! keymap
//!     // Keys can be bound individually by specifying both the category and scope.
//!     .bind("?", Action::ToggleHelp, Category::General, Scope::Global)
//!     // Sequences are supported. This binds to sequence "sg".
//!     .bind("sg", Action::SearchGrep, Category::General, Scope::Global)
//!     // "describe_group" used to add a description to groups. Display will default to "..." if
//!     // no group description is found.
//!     .describe_group("<space>", "<leader>") // (key sequence, description)
//!     .describe_group("<leader>g", "general")
//!     // Bindings can be added to a specific group while also providing a description.
//!     .group("s", "search", |g| {
//!         // "sf" binding
//!         g.bind("f", Action::SearchFiles, Category::General, Scope::SearchPanel)
//!          // "sb" binding
//!          .bind("b", Action::SearchBuffers, Category::General, Scope::SearchPanel);
//!     })
//!     // However, using `.scope` is recommended in most cases since scopes represent whatever is
//!     // currently "in focus" for your app.
//!     .scope(Scope::Global, |global| {
//!         global
//!             .bind("?", Action::ToggleHelp, Category::General)
//!             .bind("j", Action::MoveDown, Category::Navigation)
//!             // control keys supported
//!             .bind("<c-c>", Action::Quit, Category::General)
//!             // f-keys supported
//!             .bind("<F1>", Action::ToggleHelp, Category::General)
//!             // sequences supported
//!             .bind("<leader>w", Action::Save, Category::General)
//!             // sequences can start with any key
//!             .bind("gof", Action::OpenFile, Category::General);
//!     })
//!     .scope(Scope::Insert, |insert| {
//!         // While in the `Insert` scope, all keys will be routed to this handler.
//!         insert.catch_all(|key| {
//!             // You can filter the keys here
//!             match key {
//!                 CrosstermKey::Char(ch) => Some(Action::InsertModePrintableChar(ch)),
//!                 CrosstermKey::Esc => Some(Action::ToNormalMode),
//!                 _ => None
//!             }
//!         });
//!     })
//!     // Helper method if you want to bind based on category.
//!     .category(Category::Navigation, |nav| {
//!         nav
//!             .bind("k", Action::MoveUp, Scope::Global)
//!             .bind("j", Action::MoveDown, Scope::Global);
//!     })
//!     // Helper method if you want to bind based on both scope and category.
//!     .scope_and_category(Scope::Global, Category::Navigation, |g| {
//!        g.bind("<leader>gg", Action::MoveUp)
//!         .bind("<leader>gd", Action::MoveDown);
//!     });
//!
//! let app = App { which_key: WhichKeyState::new(keymap, Scope::Global) };
//!```
//!
//! # Example
//!
//! ```
//! use ratatui_which_key::{Keymap, WhichKey, WhichKeyState, CrosstermKey};
//!
//! // Define your action type
//! #[derive(Debug, Clone)]
//! enum Action { Quit, Save }
//!
//! impl std::fmt::Display for Action {
//!     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!         match self {
//!             Action::Quit => write!(f, "quit"),
//!             Action::Save => write!(f, "save"),
//!         }
//!     }
//! }
//!
//! // Define your scope type
//! #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! enum Scope { Global, Insert }
//!
//! // Define keybind categories
//! #[derive(Debug, Clone, PartialEq)]
//! enum Category { General, Navigation }
//!
//! // Build the keymap
//! let mut keymap: Keymap<CrosstermKey, Scope, Action, Category> = Keymap::new();
//! keymap.bind("q", Action::Quit, Category::General, Scope::Global);
//!
//! // Create state
//! let mut state = WhichKeyState::new(keymap, Scope::Global);
//!
//! // In your event loop, handle keys
//! # let key = CrosstermKey::Char('q');
//! if let Some(action) = state.handle_key(key).action {
//!     // dispatch action
//! }
//!
//! // When rendering
//! let widget = WhichKey::new();
//! # // (buffer from ratatui)
//! # let mut buf = ratatui::buffer::Buffer::default();
//! widget.render(&mut buf, &mut state);
//! ```
//!
//! # Feature Flags
//!
//! - `crossterm` (default): Provides `CrosstermKey` implementation

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
#[cfg(test)]
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
