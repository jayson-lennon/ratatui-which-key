// Copyright (C) 2026 Jayson Lennon
// 
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 3 of the License, or (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
// 
// You should have received a copy of the GNU Lesser General Public License
// along with this program; if not, see <https://opensource.org/license/lgpl-3-0>.

use std::sync::Arc;

use crate::{Key, Keymap};

use super::handlers::{FocusHandler, MouseHandler, ResizeHandler};

/// Storage for crossterm-specific event handlers.
pub struct CrosstermHandlers<S, A> {
    pub mouse: Option<MouseHandler<S, A>>,
    pub resize: Option<ResizeHandler<S, A>>,
    pub focus_gained: Option<FocusHandler<S, A>>,
    pub focus_lost: Option<FocusHandler<S, A>>,
}

impl<S, A> Default for CrosstermHandlers<S, A> {
    fn default() -> Self {
        Self {
            mouse: None,
            resize: None,
            focus_gained: None,
            focus_lost: None,
        }
    }
}

impl<S, A> Clone for CrosstermHandlers<S, A> {
    fn clone(&self) -> Self {
        Self {
            mouse: self.mouse.clone(),
            resize: self.resize.clone(),
            focus_gained: self.focus_gained.clone(),
            focus_lost: self.focus_lost.clone(),
        }
    }
}

/// Extension trait for Keymap to add crossterm event handlers.
pub trait CrosstermKeymapExt<K, S, A, C>: Sized
where
    K: Key,
{
    /// Register a handler for mouse events.
    #[must_use]
    fn on_mouse<F>(self, handler: F) -> Self
    where
        F: Fn(crossterm::event::MouseEvent, &S) -> Option<A> + Send + Sync + 'static;

    /// Register a handler for resize events.
    #[must_use]
    fn on_resize<F>(self, handler: F) -> Self
    where
        F: Fn(u16, u16, &S) -> Option<A> + Send + Sync + 'static;

    /// Register a handler for focus gained events.
    #[must_use]
    fn on_focus_gained<F>(self, handler: F) -> Self
    where
        F: Fn(&S) -> Option<A> + Send + Sync + 'static;

    /// Register a handler for focus lost events.
    #[must_use]
    fn on_focus_lost<F>(self, handler: F) -> Self
    where
        F: Fn(&S) -> Option<A> + Send + Sync + 'static;

    /// Get the mouse handler, if registered.
    fn mouse_handler(&self) -> Option<&MouseHandler<S, A>>;

    /// Get the resize handler, if registered.
    fn resize_handler(&self) -> Option<&ResizeHandler<S, A>>;

    /// Get the focus gained handler, if registered.
    fn focus_gained_handler(&self) -> Option<&FocusHandler<S, A>>;

    /// Get the focus lost handler, if registered.
    fn focus_lost_handler(&self) -> Option<&FocusHandler<S, A>>;
}

impl<K, S, A, C> CrosstermKeymapExt<K, S, A, C> for Keymap<K, S, A, C>
where
    K: Key,
    S: 'static,
    A: 'static,
    C: Clone,
{
    fn on_mouse<F>(mut self, handler: F) -> Self
    where
        F: Fn(crossterm::event::MouseEvent, &S) -> Option<A> + Send + Sync + 'static,
    {
        let mut handlers = self
            .backend_handlers::<CrosstermHandlers<S, A>>()
            .cloned()
            .unwrap_or_default();
        handlers.mouse = Some(Arc::new(handler));
        self.set_backend_handlers(handlers);
        self
    }

    fn on_resize<F>(mut self, handler: F) -> Self
    where
        F: Fn(u16, u16, &S) -> Option<A> + Send + Sync + 'static,
    {
        let mut handlers = self
            .backend_handlers::<CrosstermHandlers<S, A>>()
            .cloned()
            .unwrap_or_default();
        handlers.resize = Some(Arc::new(handler));
        self.set_backend_handlers(handlers);
        self
    }

    fn on_focus_gained<F>(mut self, handler: F) -> Self
    where
        F: Fn(&S) -> Option<A> + Send + Sync + 'static,
    {
        let mut handlers = self
            .backend_handlers::<CrosstermHandlers<S, A>>()
            .cloned()
            .unwrap_or_default();
        handlers.focus_gained = Some(Arc::new(handler));
        self.set_backend_handlers(handlers);
        self
    }

    fn on_focus_lost<F>(mut self, handler: F) -> Self
    where
        F: Fn(&S) -> Option<A> + Send + Sync + 'static,
    {
        let mut handlers = self
            .backend_handlers::<CrosstermHandlers<S, A>>()
            .cloned()
            .unwrap_or_default();
        handlers.focus_lost = Some(Arc::new(handler));
        self.set_backend_handlers(handlers);
        self
    }

    fn mouse_handler(&self) -> Option<&MouseHandler<S, A>> {
        self.backend_handlers::<CrosstermHandlers<S, A>>()
            .and_then(|h| h.mouse.as_ref())
    }

    fn resize_handler(&self) -> Option<&ResizeHandler<S, A>> {
        self.backend_handlers::<CrosstermHandlers<S, A>>()
            .and_then(|h| h.resize.as_ref())
    }

    fn focus_gained_handler(&self) -> Option<&FocusHandler<S, A>> {
        self.backend_handlers::<CrosstermHandlers<S, A>>()
            .and_then(|h| h.focus_gained.as_ref())
    }

    fn focus_lost_handler(&self) -> Option<&FocusHandler<S, A>> {
        self.backend_handlers::<CrosstermHandlers<S, A>>()
            .and_then(|h| h.focus_lost.as_ref())
    }
}
