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

use crossterm::event::KeyEvent;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use derive_more::Display;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use ratatui_which_key::{CrosstermKeymapExt, CrosstermStateExt, Keymap, WhichKey, WhichKeyState};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Scope {
    Normal,
    Insert,
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    General,
    Navigation,
}

#[derive(Display, Debug, Clone, PartialEq, Eq)]
enum Action {
    #[display("quit")]
    Quit,
    #[display("toggle help")]
    ToggleHelp,
    #[display("go top")]
    GoTop,
    #[display("go end")]
    GoEnd,
    #[display("go down")]
    GoDown,
    #[display("insert mode")]
    EnterInsert,
    #[display("normal mode")]
    EnterNormal,
    #[display("append")]
    Append,
    #[display("delete")]
    Delete,
    #[display("new line below")]
    NewLineBelow,
    #[display("new line above")]
    NewLineAbove,
    #[display("key: {_0}")]
    InsertModePrintableChar(char),
    #[display("mouse click at ({_0}, {_1})")]
    MouseClick(u16, u16),
    #[display("resized to {_0}x{_1}")]
    Resized(u16, u16),
    #[display("terminal focused")]
    Focused,
    #[display("terminal unfocused")]
    Unfocused,
}

#[rustfmt::skip]
fn create_keymap() -> Keymap<KeyEvent, Scope, Action, Category> {
    let mut keymap = Keymap::default();

    // Normal mode: .scope_and_category() for General bindings
    keymap.scope_and_category(Scope::Normal, Category::General, |g| {
        g.bind("q", Action::Quit)
            .bind("?", Action::ToggleHelp)
            .bind("i", Action::EnterInsert);
    });

    // Normal mode: .scope_and_category() for Navigation bindings (sequences)
    keymap.scope_and_category(Scope::Normal, Category::Navigation, |g| {
        g.bind("<leader>gg", Action::GoTop)
            .bind("<leader>ge", Action::GoEnd)
            .bind("<leader>gd", Action::GoDown);
    });

    // Insert mode: .scope() with distinct bindings (no overlap with Normal)
    keymap.scope(Scope::Insert, |insert| {
        insert
            .bind("<esc>", Action::EnterNormal, Category::General)
            .bind("a", Action::Append, Category::General)
            .bind("x", Action::Delete, Category::General)
            .bind("o", Action::NewLineBelow, Category::General)
            .bind("O", Action::NewLineAbove, Category::General)
            .bind("?", Action::ToggleHelp, Category::General)
            .catch_all(|key: KeyEvent| {
                use crossterm::event::KeyCode;
                if let KeyCode::Char(ch) = key.code {
                    Some(Action::InsertModePrintableChar(ch))
                } else {
                    None
                }
            });
    });

    keymap
        .on_mouse(|event, _scope| {
            use crossterm::event::{MouseButton, MouseEventKind};
            if let MouseEventKind::Down(MouseButton::Left) = event.kind {
                Some(Action::MouseClick(event.column, event.row))
            } else {
                None
            }
        })
        .on_resize(|cols, rows, _scope| Some(Action::Resized(cols, rows)))
        .on_focus_gained(|_scope| Some(Action::Focused))
        .on_focus_lost(|_scope| Some(Action::Unfocused))
}

struct App {
    which_key_state: WhichKeyState<KeyEvent, Scope, Action, Category>,
    position: u32,
    messages: Vec<String>,
    running: bool,
}

impl App {
    fn new() -> Self {
        let keymap = create_keymap();
        Self {
            which_key_state: WhichKeyState::new(keymap, Scope::Normal),
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
            Action::GoTop => {
                self.position = 0;
                "Went to top".to_string()
            }
            Action::GoEnd => {
                self.position = u32::MAX;
                "Went to end".to_string()
            }
            Action::GoDown => {
                self.position = self.position.saturating_add(10);
                format!("Moved down 10 to {}", self.position)
            }
            Action::EnterInsert => {
                self.which_key_state.set_scope(Scope::Insert);
                "Entered Insert mode".to_string()
            }
            Action::EnterNormal => {
                self.which_key_state.dismiss();
                self.which_key_state.set_scope(Scope::Normal);
                "Returned to Normal mode".to_string()
            }
            Action::Append => "Appended text".to_string(),
            Action::Delete => {
                format!("Deleted at position {}", self.position)
            }
            Action::NewLineBelow => "Inserted line below".to_string(),
            Action::NewLineAbove => "Inserted line above".to_string(),
            Action::InsertModePrintableChar(ch) => format!("typed: {ch}"),
            Action::MouseClick(x, y) => format!("Mouse clicked at ({x}, {y})"),
            Action::Resized(cols, rows) => format!("Terminal resized to {cols}x{rows}"),
            Action::Focused => "Terminal gained focus".to_string(),
            Action::Unfocused => "Terminal lost focus".to_string(),
        };
        self.messages.push(msg);
        if self.messages.len() > 10 {
            self.messages.remove(0);
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
        Scope::Normal => "Normal",
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
            let event = event::read()?;
            let result = app.which_key_state.handle_event(event);
            if let Some(action) = result.into_action() {
                app.handle_action(action);
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
