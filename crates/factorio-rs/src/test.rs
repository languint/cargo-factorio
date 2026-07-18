use factorio_api::LuaAny;

#[derive(Debug, Default, Clone, Copy)]
pub struct TestCtx {
    _private: (),
}

impl TestCtx {
    /// Store a value for a later step.
    pub fn set(&self, _key: &str, _value: impl Into<LuaAny>) {}

    /// Load a previously stored value.
    #[must_use]
    pub fn fetch(&self, _key: &str) -> LuaAny {
        LuaAny
    }

    /// Load a stored numeric value (e.g. a tick captured with [`Self::set`]).
    #[must_use]
    pub fn fetch_u32(&self, _key: &str) -> u32 {
        0
    }
}

/// Fluent multi-tick test builder. Created by [`steps`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Steps {
    _private: (),
}

impl Steps {
    /// Run `f` on the current tick (or immediately after a preceding wait).
    pub fn step(self, _f: impl FnOnce(&TestCtx)) -> Self {
        self
    }

    /// Advance the game by `ticks` before the next step.
    pub fn wait(self, _ticks: u32) -> Self {
        self
    }
}

/// Begin a multi-tick step sequence for the current `#[test]`.
///
/// Calling this registers a pending step queue that the harness drains across
/// ticks after the test function returns. Sync tests simply omit this call.
pub fn steps() -> Steps {
    Steps { _private: () }
}
