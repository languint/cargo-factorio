//! Factorio library deps: map Rust paths to foreign mod `require`s / remotes.

use std::collections::{BTreeMap, BTreeSet};

/// A Cargo crate that publishes `[package.metadata.factorio]` for typed interop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorioBinding {
    /// Rust crate / lib name used in `use` paths (`provider_api`).
    pub crate_name: String,
    /// Factorio internal mod name (`provider`).
    pub mod_name: String,
    /// Dependency strings contributed to the consumer's `info.json`.
    pub dependencies: Vec<String>,
    /// Require path prefix inside the Factorio mod (`lua`, or empty for mod root).
    pub module_root: String,
    /// When set, remote stub calls lower to `remote.call(interface, fn, ...)`.
    pub interface: Option<String>,
    /// Crate-root function names that lower to `remote.call` (not require).
    pub remote_fns: BTreeSet<String>,
}

/// Registry keyed by Rust crate name (`provider_api`).
pub type BindingRegistry = BTreeMap<String, FactorioBinding>;
