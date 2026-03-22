# ratatui-which-key

A which-key popup widget for [ratatui](https://github.com/ratatui/ratatui) applications, inspired by [folke's which-key.nvim](https://github.com/folke/which-key.nvim).

Displays available keybindings in a popup when the user presses a prefix key.

## How It Works

`ratatui-which-key` requires three data types be defined in your application:

- _scopes_ - this is similar to a focused element
- _actions_ - things that you want your application to do
- _categories_ - group individual bindings. This is for display purposes.

You'll set those up, store a `WhichKeyState<CrosstermKey, Scope, Action, Category>` at the top-level of your application, and set up keybinds.

Keybinds require all of the above types, along with a sequence:

```rust
let mut keymap = Keymap::new();
keymap
    .describe("<space>", "<leader>")
    .describe("<space>q", "1")
    .describe("<space>qw", "2")
    .bind("q", Action::Quit, "quit", Category::General, Scope::Global)
```

Key events are sent to `ratatui-which-key` which then uses

## Example Usage

```rust
use ratatui_which_key::{CrosstermKey, Keymap, WhichKey, WhichKeyState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scope { Global, Insert }

#[derive(Clone, Copy)]
enum Action { Quit, Save, Open }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Category { General, File }

// Build keymap
let mut keymap = Keymap::new();
keymap
    .describe("<space>", "<leader>")
    .bind("q", Action::Quit, "quit", Category::General, Scope::Global)
    .describe_prefix("f", "file", |f| {
        f.bind("s", Action::Save, "save", Category::File, Scope::Global)
         .bind("o", Action::Open, "open", Category::File, Scope::Global);
    });

// Create state
let mut state = WhichKeyState::new(keymap, Scope::Global);

// In event loop
let result = state.handle_key(crossterm_key);
if let Some(action) = result.action {
    // dispatch action
}

// In render
if state.active {
    WhichKey::new()
        .max_height(15)
        .render(frame.buffer_mut(), &mut state);
}
```

## Features

- **Tree-based keybindings** with prefix sequences (e.g., `<space>fs` for "file save")
- **Scope-based modes** (e.g., Global, Insert) with different bindings per scope
- **Customizable popup** with position, max height, and styling options
- **CrosstermKey** built-in for immediate use with crossterm-based terminals
- **Extensible** via the `Key` trait for custom key types

## License

MIT
