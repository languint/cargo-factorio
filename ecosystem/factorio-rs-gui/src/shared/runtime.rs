//! Mount / rebuild runtime for reactive GUIs.

use factorio_rs::{
    factorio_api::{LuaFunction, classes::LuaGuiElement, concepts::GuiLocation},
    prelude::*,
};

use super::widget::Widget;

pub(crate) const RT_PARENT: &str = "frg_parent";
pub(crate) const RT_APP: &str = "frg_app";
pub(crate) const RT_HOOK_I: &str = "frg_hook_i";
pub(crate) const RT_HOOK_N: &str = "frg_hook_n";
pub(crate) const RT_DIRTY: &str = "frg_dirty";
pub(crate) const RT_NEXT_ID: &str = "frg_next_id";
pub(crate) const RT_HANDLERS: &str = "frg_handlers";
pub(crate) const RT_LOCATION: &str = "frg_location";

/// Click handler binding (element name -> Lua callback).
pub struct ClickBinding {
    pub name: String,
    pub handler: LuaFunction,
}

/// Handle to a hook slot created by [`state!`](crate::state) / [`State::use_state`].
///
/// Instances are built only inside [`State`] methods so Lua tables carry the
/// method metatable (`get` / `set`) across mod boundaries.
pub struct State {
    pub(crate) index: i32,
}

impl State {
    /// Hook-style state: stable across rebuilds, ordered by call site.
    #[must_use]
    pub fn use_state(init: i32) -> Self {
        let idx = storage.get::<i32>(RT_HOOK_I).unwrap_or(0);
        let key = hook_key(idx);
        if storage.get::<i32>(&key).is_none() {
            storage.set(&key, init);
        }
        storage.set(RT_HOOK_I, idx + 1);
        let n = storage.get::<i32>(RT_HOOK_N).unwrap_or(0);
        if idx + 1 > n {
            storage.set(RT_HOOK_N, idx + 1);
        }
        Self { index: idx }
    }

    /// Read the current value.
    #[must_use]
    pub fn get(&self) -> i32 {
        let key = hook_key(self.index);
        storage.get::<i32>(&key).unwrap_or(0)
    }

    /// Write a new value and schedule a GUI rebuild.
    pub fn set(&self, value: i32) {
        let key = hook_key(self.index);
        storage.set(&key, value);
        storage.set(RT_DIRTY, true);
        flush_dirty();
    }
}

/// Storage key for a hook slot.
#[must_use]
#[factorio_rs::inline]
pub fn hook_key(index: i32) -> String {
    format!("frg_hook_{index}")
}

/// Mount a reactive app under `parent`.
///
/// `app` is re-invoked on every rebuild (state change). Prefer passing a
/// function item or `lua_fn0(|| { ... })`.
#[factorio_rs::inline]
pub fn mount(parent: LuaGuiElement, app: LuaFunction) {
    storage.set(RT_PARENT, parent);
    storage.set(RT_APP, app);
    storage.set(RT_HOOK_I, 0_i32);
    storage.set(RT_HOOK_N, 0_i32);
    storage.set(RT_DIRTY, false);
    storage.set(RT_NEXT_ID, 0_i32);
    storage.set(RT_HANDLERS, Vec::<ClickBinding>::new());
    rebuild();
}

/// Rebuild the tree: reset hooks cursor, clear children, call `app`, mount.
///
/// The root element's screen [`GuiLocation`] is snapshotted before `clear` and
/// reapplied after remount so dragged windows stay put across state updates.
#[factorio_rs::inline]
pub fn rebuild() {
    storage.set(RT_HOOK_I, 0_i32);
    storage.set(RT_HANDLERS, Vec::<ClickBinding>::new());
    storage.set(RT_DIRTY, false);
    storage.set(RT_NEXT_ID, 0_i32);

    if let Some(parent) = storage.get::<LuaGuiElement>(RT_PARENT) {
        let existing = parent.children();
        if !existing.is_empty() {
            // Lua `nil` clears the key; a real `{x,y}` persists for restore.
            storage.set(RT_LOCATION, existing[0].location());
        }

        parent.clear();
        if let Some(app) = storage.get::<LuaFunction>(RT_APP) {
            let root = app.invoke0::<Widget>();
            root.mount(parent);
            if let Some(loc) = storage.get::<GuiLocation>(RT_LOCATION) {
                let mounted = parent.children();
                if !mounted.is_empty() {
                    mounted[0].set_location(loc);
                    mounted[0].set_auto_center(false);
                }
            }
        }
    }
}

/// Allocate a unique element name for click routing.
#[must_use]
#[factorio_rs::inline]
pub fn next_element_name(prefix: &str) -> String {
    let id = storage.get::<i32>(RT_NEXT_ID).unwrap_or(0);
    storage.set(RT_NEXT_ID, id + 1);
    format!("{prefix}_{id}")
}

/// Register a click handler for an element name.
#[factorio_rs::inline]
pub fn register_click(name: String, handler: LuaFunction) {
    let mut handlers = match storage.get::<Vec<ClickBinding>>(RT_HANDLERS) {
        Some(handlers) => handlers,
        None => Vec::new(),
    };
    handlers.push(ClickBinding { name, handler });
    storage.set(RT_HANDLERS, handlers);
}

/// Dispatch `OnGuiClick` to a registered handler, then rebuild if dirty.
///
/// Call this from **your** mod's `OnGuiClick` handler. Factorio gives each mod
/// its own `storage`, so a library-mod event cannot see handlers registered
/// while your mod was mounting the GUI.
#[factorio_rs::inline]
pub fn dispatch_click(event: OnGuiClickEvent) {
    let name = event.element.name();
    if let Some(handlers) = storage.get::<Vec<ClickBinding>>(RT_HANDLERS) {
        for binding in handlers {
            if binding.name == name {
                binding.handler.invoke(event);
            }
        }
    }
    flush_dirty();
}

/// If state changed during a handler, rebuild now.
#[factorio_rs::inline]
pub fn flush_dirty() {
    if storage.get::<bool>(RT_DIRTY).unwrap_or(false) {
        rebuild();
    }
}
