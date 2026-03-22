use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
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
    EnterInsert,
    EnterNormal,
    NewLine,
    GoLeft,
    GoRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Global,
    Insert,
}

#[rustfmt::skip]
fn create_keymap() -> Keymap<CrosstermKey, Scope, Action, Category> {
    let mut keymap = Keymap::new();

    // Normal mode: .scope_and_category() for General bindings
    keymap.scope_and_category(Scope::Global, Category::General, |g| {
        g.bind("q", Action::Quit, "quit")
            .bind("?", Action::ToggleHelp, "show help")
            .bind("i", Action::EnterInsert, "enter insert mode");
    });

    // Normal mode: .scope_and_category() for Navigation bindings
    keymap.scope_and_category(Scope::Global, Category::Navigation, |g| {
        g.bind("k", Action::MoveUp, "move up")
            .bind("j", Action::MoveDown, "move down")
            .bind("h", Action::GoLeft, "go left")
            .bind("l", Action::GoRight, "go right");
    });

    // Insert mode: .scope() with explicit categories - bindings share scope but differ in category
    keymap.scope(Scope::Insert, |insert| {
        insert
            .bind("<esc>", Action::EnterNormal, "exit insert mode", Category::General)
            .bind("x", Action::Delete, "delete char", Category::General)
            .bind("<enter>", Action::NewLine, "insert newline", Category::General)
            .bind("h", Action::GoLeft, "move left", Category::Navigation)
            .bind("l", Action::GoRight, "move right", Category::Navigation)
            .bind("k", Action::MoveUp, "move up", Category::Navigation)
            .bind("j", Action::MoveDown, "move down", Category::Navigation);
    });

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
                format!("Moved up to {}", self.position)
            }
            Action::MoveDown => {
                self.position = self.position.saturating_add(1);
                format!("Moved down to {}", self.position)
            }
            Action::GoLeft => format!("Moved left (pos={})", self.position),
            Action::GoRight => format!("Moved right (pos={})", self.position),
            Action::Save => "File saved".to_string(),
            Action::OpenFile => "File opened".to_string(),
            Action::Delete => format!("Deleted at position {}", self.position),
            Action::EnterInsert => {
                self.which_key_state.set_scope(Scope::Insert);
                "Entered Insert mode (press <esc> to exit)".to_string()
            }
            Action::EnterNormal => {
                self.which_key_state.set_scope(Scope::Global);
                "Returned to Normal mode".to_string()
            }
            Action::NewLine => "New line inserted".to_string(),
        };
        self.messages.push(msg);
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }

    fn handle_key(&mut self, key: CrosstermKey) {
        if let Some(action) = self.which_key_state.handle_key(key).action {
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
    let mode = match app.which_key_state.scope() {
        Scope::Global => "Normal",
        Scope::Insert => "Insert",
    };
    lines.push(Line::styled(
        format!("Mode: {mode} | Press ? to show keybindings, q to quit"),
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
