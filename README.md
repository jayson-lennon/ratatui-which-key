# ratatui-which-key

A which-key popup widget + input handler for [ratatui](https://github.com/ratatui/ratatui) applications, inspired by [folke's which-key.nvim](https://github.com/folke/which-key.nvim).

![screenshot 1](media/screenshot-1.png)
![screenshot 2](media/screenshot-2.png)

All input can be routed to `ratatui-which-key` and it will return an application-specific action to perform based on the configured keybinds.

## How It Works

`ratatui-which-key` requires three data types be defined in your application.

### Scopes

The _scope_ is what part of your application is currently "in focus":

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Global,
    TextInputBox,
    SearchPanel,
    // ....
}

// When changing focus to another pane/window/etc:
app.which_key.set_scope(Scope::TextInputBox)
```

### Actions

`ratatui-which-key` returns an `Action` when a keybind is triggered:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Quit,
    ToggleHelp,
    MoveUp,
    MoveDown,
    Save,
    OpenFile,
    SearchFiles,
    SearchBuffers,
    // ...
}

// Must implement Display to show descriptions in the which-key popup.
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Quit => write!(f, "quit"),
            Action::ToggleHelp => write!(f, "toggle help"),
            Action::MoveUp => write!(f, "") => write!(f, "move up")
            Action::MoveDown => write!(f, "move down"),
            Action::Save => write!(f, "save"),
            Action::OpenFile => write!(f, "open file"),
            Action::SearchFiles => write!(f, "search files"),
            Action::SearchBuffers => write!(f, "search buffers"),
        }
    }
}

// In your input handler:
if let Some(action) = app.which_key.handle_key(key).action {
    match action {
        Action::Quit => // ...
        Action::ToggleHelp => app.which_key.toggle(),
        Action::MoveUp => // ...
        Action::MoveDown = // ...
        Action::Save = // ...
        Action::OpenFile = // ...
        Action::SearchFiles = // ...
        Action::SearchBuffers = // ...
    }
}
```

### Categories

The `ratatui-which-key` popup displays keybinds sorted by category:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    General,
    Navigation,
    Search,
    // ...
}
```

## Keymap Configuration

You'll need to put a `WhichKeyState<CrosstermKey, Scope, Action, Category>` at the top-level of your application (like in `App`). Then at program start, configure your keybinds by creating a new `Keymap`:

```rust
struct App {
    which_key: WhichKeyState<CrosstermKey, Scope, Action, Category>,
}

let mut keymap = Keymap::new();
keymap
    // "describe_group" is a way to set the name of a group explicitly
    .describe_group("<space>", "<leader>")
    // keys can be bound individually
    .bind("?", Action::ToggleHelp, Category::General, Scope::Global)
    .bind("j", Action::MoveDown, Category::Navigation, Scope::Global)
    // control keys supported
    .bind("<c-c>", Action::Quit, Category::General, Scope::Global)
    // f-keys supported
    .bind("<F1>", Action::ToggleHelp, Category::General, Scope::Global)
    // sequences supported
    .bind("<leader>w", Action::Save, Category::General, Scope::Global)
    // sequences can start with any key
    .bind("gof", Action::OpenFile, Category::General, Scope::Global)
    // group configuration by prefix. No need to use `describe_group` when using this method.
    .group("s", "search", |g| {
        // bind to `sf`
        g.bind("f", Action::SearchFiles, Category::General, Scope::SearchPanel)
        // bind to `sb`
         .bind("b", Action::SearchBuffers, Category::General, Scope::SearchPanel);
     })
     // automated scope association. No need to specify scope for each binding.
     .scope(Scope::Global, |global| {
         global
             .bind("?", Action::ToggleHelp, Category::General)
             .bind("j", Action::MoveDown, Category::Navigation);
     })
     // automated category association. No need to specify category for each binding.
     .category(Category::Navigation, |nav| {
         nav
             .bind("k", Action::MoveUp, Scope::Global)
             .bind("j", Action::MoveDown, Scope::Global);
     })
     // automated scope + category association.
     .scope_and_category(Scope::Global, Category::Navigation, |g| {
        g.bind("<leader>gg", Action::MoveUp)
         .bind("<leader>gd", Action::MoveDown);
     });

// Create new state with a keymap and initial scope.
app.which_key_state = WhichKeyState::new(keymap, Scope::Global);
```

Finally, to render:

```rust
// (in your top-level render function)
if app.which_key.active {
    let widget = WhichKey::new().border_style(Style::default().fg(Color::Green));
    widget.render(frame.buffer_mut(), &mut app.which_key);
}
```

There is a [sample application](examples/demo.rs) that you can run with `cargo run --example demo` which shows how to perform bindings and set up `ratatui-which-key` for usage in an app.

## License

AGPLv3
