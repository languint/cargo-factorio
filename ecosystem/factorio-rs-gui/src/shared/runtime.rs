use factorio_rs::{
    factorio_api::{LuaFunction, classes::LuaGuiElement, concepts::GuiLocation},
    prelude::*,
};

use super::widget::Widget;

/// Default root name for single-GUI mods. Prefer a mod-unique string in
/// [`mount`] when more than one GUI may share the same parent.
pub const ROOT_NAME: &str = "frg_root";

const RT_CURRENT: &str = "frg_current_root";

/// Click handler binding (element name -> Lua callback).
pub struct ClickBinding {
    pub name: String,
    pub handler: LuaFunction,
}

struct GuiSession {
    root: String,
    app: Option<LuaFunction>,
    handlers: Vec<ClickBinding>,
}

const SESSIONS: Vec<GuiSession> = Vec::new();

struct EventsGate {
    bound: bool,
}

const EVENTS_GATE: EventsGate = EventsGate { bound: false };

const EXTRA_CLICK_HANDLERS: Vec<LuaFunction> = Vec::new();

/// Handle to a hook slot created by [`state!`](crate::state) / [`State::use_state`].
///
/// Instances are built only inside [`State`] methods so Lua tables carry the
/// method metatable (`get` / `set`) across mod boundaries.
pub struct State {
    pub(crate) root: String,
    pub(crate) index: i32,
}

impl State {
    /// Hook-style state: stable across rebuilds, ordered by call site.
    ///
    /// Hooks are namespaced by the root name of the GUI currently being built
    /// (see [`mount`] / [`rebuild_root`]).
    #[must_use]
    pub fn use_state(init: i32) -> Self {
        let root = current_root();
        let idx_key = sk(&root, "hook_i");
        let n_key = sk(&root, "hook_n");
        let idx = storage.get::<i32>(&idx_key).unwrap_or(0);
        let key = hook_key(&root, idx);
        if storage.get::<i32>(&key).is_none() {
            storage.set(&key, init);
        }
        storage.set(&idx_key, idx + 1);
        let n = storage.get::<i32>(&n_key).unwrap_or(0);
        if idx + 1 > n {
            storage.set(&n_key, idx + 1);
        }
        Self { root, index: idx }
    }

    /// Read the current value.
    #[must_use]
    pub fn get(&self) -> i32 {
        let key = hook_key(&self.root, self.index);
        storage.get::<i32>(&key).unwrap_or(0)
    }

    /// Write a new value and schedule a GUI rebuild for this root.
    pub fn set(&self, value: i32) {
        let key = hook_key(&self.root, self.index);
        storage.set(&key, value);
        storage.set(&sk(&self.root, "dirty"), true);
        flush_dirty(&self.root);
    }
}

/// Storage key prefix for one mounted GUI.
#[must_use]
#[factorio_rs::inline]
fn sk(root: &str, key: &str) -> String {
    format!("frg:{root}:{key}")
}

/// Storage key for a hook slot under `root`.
#[must_use]
#[factorio_rs::inline]
pub fn hook_key(root: &str, index: i32) -> String {
    format!("frg:{root}:hook_{index}")
}

#[factorio_rs::inline]
fn current_root() -> String {
    storage
        .get::<String>(RT_CURRENT)
        .unwrap_or_else(|| ROOT_NAME.into())
}

/// Find a direct child of `parent` by element name.
#[factorio_rs::inline]
fn find_named_child(parent: LuaGuiElement, name: &str) -> Option<LuaGuiElement> {
    for child in parent.children() {
        if child.name() == name {
            return Some(child);
        }
    }
    None
}

/// Snapshot drag location from a screen frame, if it has been moved.
#[factorio_rs::inline]
fn snapshot_root_location(root_name: &str, root: LuaGuiElement) {
    if root.r#type() != "frame" {
        return;
    }
    if !root.auto_center() {
        storage.set(&sk(root_name, "location"), root.location());
    }
}

/// Ensure a [`GuiSession`] exists for `root_name` and install `app`.
#[factorio_rs::inline]
fn bind_app(root_name: &str, app: LuaFunction) {
    let sessions = SESSIONS;
    for mut session in sessions {
        if session.root == root_name {
            session.app = Some(app);
            session.handlers = Vec::new();
            return;
        }
    }
    // Lua: `SESSIONS` is a shared table; push mutates it even though Rust
    // sees the local binding as unread after the call.
    #[allow(clippy::collection_is_never_read)]
    {
        let mut sessions = SESSIONS;
        sessions.push(GuiSession {
            root: root_name.into(),
            app: Some(app),
            handlers: Vec::new(),
        });
    }
}

/// Clear click handlers for `root_name` before a rebuild.
#[factorio_rs::inline]
fn clear_handlers(root_name: &str) {
    let sessions = SESSIONS;
    for mut session in sessions {
        if session.root == root_name {
            session.handlers = Vec::new();
            return;
        }
    }
}

/// Look up the app closure for `root_name`.
#[factorio_rs::inline]
fn session_app(root_name: &str) -> Option<LuaFunction> {
    let sessions = SESSIONS;
    for session in sessions {
        if session.root == root_name {
            return session.app;
        }
    }
    None
}

/// Whether this Lua session already has an app bound for `root_name`.
#[factorio_rs::inline]
fn session_has_app(root_name: &str) -> bool {
    session_app(root_name).is_some()
}

#[factorio_rs::inline]
pub fn ensure_events() {
    let mut gate = EVENTS_GATE;
    if gate.bound {
        return;
    }
    #[allow(unused_assignments)]
    {
        gate.bound = true;
    }
    script.on_event(
        LuaEventType::Name("on_gui_click"),
        dispatch_click as fn(OnGuiClickEvent),
        None,
    );
}

/// Mount a reactive app under `parent`.
///
/// `app` is re-invoked on every rebuild (state change). Prefer passing a
/// function item or `lua_fn0(|| { ... })`.
#[factorio_rs::inline]
pub fn mount(parent: LuaGuiElement, root_name: &str, app: LuaFunction) {
    ensure_events();
    storage.set(RT_CURRENT, root_name.to_string());
    storage.set(&sk(root_name, "parent"), parent);
    storage.set(&sk(root_name, "hook_i"), 0_i32);
    storage.set(&sk(root_name, "hook_n"), 0_i32);
    storage.set(&sk(root_name, "dirty"), false);
    storage.set(&sk(root_name, "next_id"), 0_i32);
    // Fresh mount: drop any dragged location so centering can apply.
    storage.set(&sk(root_name, "location"), None::<GuiLocation>);

    bind_app(root_name, app);
    rebuild_root(root_name);
}

#[factorio_rs::inline]
pub fn install(root_name: &str, app: LuaFunction) {
    ensure_events();
    restore(root_name, app);
}

/// Rebuild every mounted root that still has a parent and app.
///
/// Prefer [`rebuild_root`] when only one GUI changed.
#[factorio_rs::inline]
pub fn rebuild() {
    let sessions = SESSIONS;
    for session in sessions {
        let root = session.root.clone();
        if session.app.is_some() && storage.get::<LuaGuiElement>(&sk(&root, "parent")).is_some() {
            rebuild_root(&root);
        }
    }
}

#[factorio_rs::inline]
pub fn rebuild_root(root_name: &str) {
    storage.set(RT_CURRENT, root_name.to_string());
    storage.set(&sk(root_name, "hook_i"), 0_i32);
    storage.set(&sk(root_name, "dirty"), false);
    storage.set(&sk(root_name, "next_id"), 0_i32);
    clear_handlers(root_name);

    let parent_key = sk(root_name, "parent");
    if let Some(parent) = storage.get::<LuaGuiElement>(&parent_key) {
        if let Some(existing) = find_named_child(parent, root_name) {
            snapshot_root_location(root_name, existing);
            existing.destroy();
        }

        if let Some(app) = session_app(root_name) {
            let root = app.invoke0::<Widget>().with_root_name(root_name);
            root.mount(parent);
            if let Some(loc) = storage.get::<GuiLocation>(&sk(root_name, "location"))
                && let Some(mounted) = find_named_child(parent, root_name)
                && mounted.r#type() == "frame"
            {
                mounted.set_location(loc);
                mounted.set_auto_center(false);
            }
        }
    }
}

/// Re-bind `app` for `root_name` and rebuild after a script / mod reload.
#[factorio_rs::inline]
pub fn restore(root_name: &str, app: LuaFunction) {
    if session_has_app(root_name) {
        return;
    }
    if storage
        .get::<LuaGuiElement>(&sk(root_name, "parent"))
        .is_none()
    {
        return;
    }
    bind_app(root_name, app);
    rebuild_root(root_name);
}

/// Destroy the named root frame and drop its app / handlers.
///
/// Hook values in `storage` are left in place so a later [`mount`] with the
/// same name can restore them.
#[factorio_rs::inline]
pub fn unmount(root_name: &str) {
    if let Some(parent) = storage.get::<LuaGuiElement>(&sk(root_name, "parent"))
        && let Some(existing) = find_named_child(parent, root_name)
    {
        existing.destroy();
    }
    storage.set(&sk(root_name, "parent"), None::<LuaGuiElement>);
    storage.set(&sk(root_name, "location"), None::<GuiLocation>);
    storage.set(&sk(root_name, "dirty"), false);

    let sessions = SESSIONS;
    for mut session in sessions {
        if session.root == root_name {
            session.app = None;
            session.handlers = Vec::new();
            return;
        }
    }
}

/// Allocate a unique element name for click routing under the current root.
#[must_use]
#[factorio_rs::inline]
pub fn next_element_name(prefix: &str) -> String {
    let root = current_root();
    let key = sk(&root, "next_id");
    let id = storage.get::<i32>(&key).unwrap_or(0);
    storage.set(&key, id + 1);
    format!("{root}:{prefix}_{id}")
}

/// Register a click handler for an element name under the current root.
#[factorio_rs::inline]
pub fn register_click(name: String, handler: LuaFunction) {
    let root = current_root();
    let sessions = SESSIONS;
    for mut session in sessions {
        if session.root == root {
            session.handlers.push(ClickBinding { name, handler });
            return;
        }
    }
}

#[factorio_rs::inline]
pub fn on_click(handler: LuaFunction) {
    ensure_events();
    #[allow(clippy::collection_is_never_read)]
    {
        let mut extras = EXTRA_CLICK_HANDLERS;
        extras.push(handler);
    }
}

/// Dispatch `OnGuiClick` to a registered handler, then rebuild if dirty.
///
/// Prefer [`install`] / [`mount`], which register this via `script.on_event`.
/// You normally do not need a manual `OnGuiClick` stub. For additional click
/// logic, use [`on_click`].
#[factorio_rs::inline]
pub fn dispatch_click(event: OnGuiClickEvent) {
    let name = event.element.name();
    let sessions = SESSIONS;
    for session in sessions {
        let root = session.root.clone();
        for binding in session.handlers {
            if binding.name == name {
                binding.handler.invoke(event);
                flush_dirty(&root);
                return;
            }
        }
    }

    let extras = EXTRA_CLICK_HANDLERS;
    for handler in extras {
        handler.invoke(event);
    }
}

/// If state changed during a handler for `root_name`, rebuild now.
#[factorio_rs::inline]
pub fn flush_dirty(root_name: &str) {
    if storage
        .get::<bool>(&sk(root_name, "dirty"))
        .unwrap_or(false)
    {
        rebuild_root(root_name);
    }
}
