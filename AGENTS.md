# Style Guide

This document defines the coding conventions and architectural patterns for the ratatui-which-key codebase.

## 1. Overview

This style guide ensures consistent, maintainable Rust code across the codebase. It covers error handling, trait-based design, testing patterns, documentation standards, and module organization. Following these patterns enables dependency injection for testability and clear separation of concerns.

## 2. Tests

Important:

- Tests should only verify _observable behavior_
- Testing internal details is an _anti-pattern_.
- Prefer testing observable behavior ONLY. If observable behavior cannot be tested, then an abstraction needs to be created. Ask the user how to proceed in this case.

### BDD-Style Tests (Given/When/Then)

Structure tests with clear Given/When/Then sections:

```rust
fn pop_returns_none_when_stack_empty() {
    // Given an empty stack.
    let mut stack = Stack::default();

    // When popping from the stack.
    let item = stack.pop();

    // Then we get nothing back.
    assert!(item.is_none());
}
```

**Example with service:**

```rust
fn service_delegates_to_backend() {
    // Given a service with a fake backend.
    let fake = Arc::new(FakeBackend::new());
    let service = MyService::new(fake.clone());

    // When calling the service method.
    let result = service.do_thing();

    // Then the backend was called and result is successful.
    assert!(result.is_ok());
    assert_eq!(fake.call_count.load(Ordering::SeqCst), 1);
}
```

### Test Utilities

**test_utils module structure:**

```rust
// test_utils/mod.rs
pub mod context;
pub mod fakes;
pub mod fixtures;
pub mod services;

pub use context::NoteTestContext;
pub use fakes::FakeMpvBackend;
pub use services::create_test_services;
```

**Test context pattern:**

```rust
pub struct NoteTestContext {
    pub ctx: SystemCtx,
    pub temp_file: NamedTempFile,
}

impl NoteTestContext {
    pub async fn new() -> Self {
        let services = create_test_services().await;
        let ctx = SystemCtx { services, ... };
        Self { ctx, temp_file }
    }
}
```

**Test services factory:**

```rust
pub async fn create_test_services() -> Services {
    let db = Arc::new(SqliteNoteDb::new("sqlite::memory:").await.unwrap());
    Services {
        mpv: MpvClientService::new(Arc::new(FakeMpvBackend)),
        media: MediaQueryService::new(Arc::new(FakeMediaBackend)),
        // ... all services with fakes
    }
}
```

### Fake Implementations

**Simple fake:**

```rust
pub struct FakeMpvBackend;

impl MpvClient for FakeMpvBackend {
    fn name(&self) -> &'static str { "fake" }
    fn load_file(&self, _path: &Path) -> Result<(), Report<MpvError>> {
        Ok(())
    }
}
```

**Stateful fake with call tracking:**

```rust
pub struct FakeStorageBackend {
    data: Arc<RwLock<StorageData>>,
    pub load_called: AtomicUsize,
}

impl FakeStorageBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(StorageData::default())),
            load_called: AtomicUsize::new(0),
        }
    }
}

impl PlaylistStorage for FakeStorageBackend {
    async fn load(&self, _dir: &CanonicalPath) -> Result<PlaylistData, Report<IoError>> {
        self.load_called.fetch_add(1, Ordering::SeqCst);
        Ok(self.data.read().await.clone())
    }
}
```

## 3. Tooling

Read the `justfile` to determine what additional tooling is related to this project. Prioritize running commands from the `justfile` instead of manual invocation. If there is a `just test` command, then use that instead of `cargo test`, etc.

## 4. Misc

- NEVER manually split a string using `.chars` or by indexing. Use the `unicode-segmentation` crate.
