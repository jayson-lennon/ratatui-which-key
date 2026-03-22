use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use ratatui_which_key::{CrosstermKey, Keymap, WhichKey, WhichKeyState};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    General,
    Navigation,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Quit,
    ToggleHelp,
    MoveUp,
    MoveDown,
    Save,
    OpenFile,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Global,
    Insert,
}

#[rustfmt::skip]
fn create_keymap() -> Keymap<CrosstermKey, Scope, Action, Category> {
    let mut keymap = Keymap::new();

    keymap
        .describe("<space>", "<leader>")
        .describe("<space>q", "1")
        .describe("<space>qw", "2")
        .bind("q", Action::Quit, "quit", Category::General, Scope::Global)
        .bind( "<leader>qwe", Action::MoveUp, "nested", Category::General, Scope::Global,)
        .bind( "?", Action::ToggleHelp, "show help", Category::General, Scope::Global,)
        .bind( "<F1>", Action::ToggleHelp, "show help", Category::General, Scope::Global,)
        .bind( "k", Action::MoveUp, "move up", Category::Navigation, Scope::Global,)
        .bind( "j", Action::MoveDown, "move down", Category::Navigation, Scope::Global,)
        .describe_prefix("g", "goto", |g| {
            g.bind( "g", Action::MoveUp, "go to top", Category::Navigation, Scope::Global,)
             .bind( "e", Action::MoveDown, "go to end", Category::Navigation, Scope::Global,);
        })
        .describe_prefix("f", "file", |f| {
            f.bind( "s", Action::Save, "save file", Category::General, Scope::Global,)
             .bind( "o", Action::OpenFile, "open file", Category::General, Scope::Global,);
        })
        .bind( "<esc>", Action::Quit, "exit insert mode", Category::General, Scope::Insert,)
        .bind( "<enter>", Action::Quit, "new line", Category::General, Scope::Insert,);

    keymap
}

struct App {
    which_key_state: WhichKeyState<CrosstermKey, Scope, Action, Category>,
    position: u32,
    messages: Vec<String>,
    running: bool,
}

impl App {
    fn new() -> Self {
        let keymap = create_keymap();
        Self {
            which_key_state: WhichKeyState::new(keymap, Scope::Global),
            position: 0,
            messages: vec!["Press ? to show keybindings".to_string()],
            running: true,
        }
    }

    fn handle_action(&mut self, action: Action) {
        let msg = match action {
            Action::Quit => {
                self.running = false;
                return;
            }
            Action::ToggleHelp => {
                self.which_key_state.toggle();
                "Help toggled".to_string()
            }
            Action::MoveUp => {
                self.position = self.position.saturating_sub(1);
                format!("Moved up to position {}", self.position)
            }
            Action::MoveDown => {
                self.position = self.position.saturating_add(1);
                format!("Moved down to position {}", self.position)
            }
            Action::Save => "File saved".to_string(),
            Action::OpenFile => "File opened".to_string(),
            Action::Delete => "Item deleted".to_string(),
        };
        self.messages.push(msg);
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }

    fn handle_key(&mut self, key: CrosstermKey) {
        let result = self.which_key_state.handle_key(key);
        if let Some(action) = result.action {
            self.handle_action(action);
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let block = Block::default()
        .title("ratatui-which-key Demo")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![];
    lines.push(Line::raw(format!("Position: {}", app.position)));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Messages:",
        Style::default().fg(Color::Yellow),
    ));
    for msg in &app.messages {
        lines.push(Line::raw(format!("  {msg}")));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Press ? to show keybindings, q to quit",
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    if app.which_key_state.active {
        let widget = WhichKey::new().border_style(Style::default().fg(Color::Green));
        widget.render(frame.buffer_mut(), &mut app.which_key_state);
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    while app.running {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let Some(crossterm_key) = CrosstermKey::from_keycode(key.code, key.modifiers) {
                    app.handle_key(crossterm_key);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
