//! Example demonstrating extension traits for ScopeBuilder.
//!
//! This pattern allows users to create convenience methods that pre-fill
//! the category parameter for their custom category types.

use ratatui_which_key::{CrosstermKey, Keymap, ScopeBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    General,
    Navigation,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Quit,
    Save,
    MoveUp,
    MoveDown,
    DeleteChar,
    InsertMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    Global,
    Insert,
}

trait ScopeBuilderExt<K, S, A>: Sized {
    fn bind_general(&mut self, key: &str, action: A, description: &'static str) -> &mut Self;
    fn bind_navigation(&mut self, key: &str, action: A, description: &'static str) -> &mut Self;
    fn bind_editing(&mut self, key: &str, action: A, description: &'static str) -> &mut Self;
}

impl<K, S, A> ScopeBuilderExt<K, S, A> for ScopeBuilder<'_, K, S, A, Category>
where
    K: ratatui_which_key::Key + Clone,
    S: Clone + PartialEq,
    A: Clone,
{
    fn bind_general(&mut self, key: &str, action: A, description: &'static str) -> &mut Self {
        self.bind(key, action, description, Category::General)
    }

    fn bind_navigation(&mut self, key: &str, action: A, description: &'static str) -> &mut Self {
        self.bind(key, action, description, Category::Navigation)
    }

    fn bind_editing(&mut self, key: &str, action: A, description: &'static str) -> &mut Self {
        self.bind(key, action, description, Category::Editing)
    }
}

fn main() {
    let mut keymap: Keymap<CrosstermKey, Scope, Action, Category> = Keymap::new();

    keymap.scope(Scope::Global, |global| {
        global
            .bind_general("q", Action::Quit, "quit")
            .bind_general("s", Action::Save, "save")
            .bind_navigation("j", Action::MoveDown, "move down")
            .bind_navigation("k", Action::MoveUp, "move up");
    });

    keymap.scope(Scope::Insert, |insert| {
        insert
            .bind_editing("<esc>", Action::InsertMode, "exit insert mode")
            .bind_editing("<backspace>", Action::DeleteChar, "delete char");
    });

    println!("Keymap created with extension trait convenience methods");
}
