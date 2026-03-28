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

/// Handler for mouse events.
pub type MouseHandler<S, A> =
    Arc<dyn Fn(crossterm::event::MouseEvent, &S) -> Option<A> + Send + Sync>;

/// Handler for resize events.
pub type ResizeHandler<S, A> = Arc<dyn Fn(u16, u16, &S) -> Option<A> + Send + Sync>;

/// Handler for focus events.
pub type FocusHandler<S, A> = Arc<dyn Fn(&S) -> Option<A> + Send + Sync>;
