//! Export manifests and ephemeral binding stub crates for `#[factorio_rs::export]`.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    cargo_manifest::CargoPackage,
    error::{CliError, CliResult},
    manifest::{RemoteExport, SharedConst, SharedExport},
};

/// Provider-side export catalog written to `.factorio-rs/exports.json`.
pub const EXPORTS_MANIFEST_REL: &str = ".factorio-rs/exports.json";
/// Consumer-side library path registry.
pub const LIBRARIES_REGISTRY_REL: &str = ".factorio-rs/libraries.json";
/// Ephemeral stub crates live under this consumer-relative directory.
pub const BINDINGS_DIR_REL: &str = "target/factorio-rs/bindings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportsManifest {
    pub mod_name: String,
    pub version: String,
    pub dependency: String,
    pub module_root: String,
    /// Primary remote interface (first remote export, or mod name).
    pub interface: String,
    pub remotes: Vec<ManifestRemote>,
    pub shared_fns: Vec<ManifestSharedFn>,
    pub shared_consts: Vec<ManifestSharedConst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestRemote {
    pub function: String,
    pub interface: String,
    pub params: Vec<ManifestParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSharedFn {
    pub module: String,
    pub function: String,
    pub params: Vec<ManifestParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSharedConst {
    pub module: String,
    pub name: String,
    pub source_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestParam {
    pub name: String,
    pub ty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LibrariesRegistry {
    /// mod_name → path to the library project (relative to consumer root).
    #[serde(default)]
    pub libraries: BTreeMap<String, String>,
}

/// Write `{project}/.factorio-rs/exports.json` when there is anything to export.
///
/// # Errors
/// Returns I/O or serialize errors.
pub fn write_exports_manifest(
    project_root: &Path,
    package: &CargoPackage,
    remote_exports: &[RemoteExport],
    shared_exports: &[SharedExport],
    shared_consts: &[SharedConst],
) -> CliResult<Option<PathBuf>> {
    if remote_exports.is_empty() && shared_exports.is_empty() && shared_consts.is_empty() {
        return Ok(None);
    }

    let manifest = build_manifest(package, remote_exports, shared_exports, shared_consts);
    let path = project_root.join(EXPORTS_MANIFEST_REL);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CliError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let json =
        serde_json::to_string_pretty(&manifest).map_err(|source| CliError::CargoMetadata {
            message: format!("failed to serialize exports manifest: {source}"),
        })?;
    std::fs::write(&path, format!("{json}\n")).map_err(|source| CliError::WriteFile {
        path: path.clone(),
        source,
    })?;
    Ok(Some(path))
}

/// Load an exports manifest from a library project root.
///
/// # Errors
/// Missing file → [`CliError::ExportsManifestMissing`]; other I/O/parse errors.
pub fn load_exports_manifest(lib_root: &Path) -> CliResult<ExportsManifest> {
    let path = lib_root.join(EXPORTS_MANIFEST_REL);
    if !path.exists() {
        return Err(CliError::ExportsManifestMissing { path });
    }
    let contents = std::fs::read_to_string(&path).map_err(|source| CliError::ReadFile {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| CliError::CargoMetadata {
        message: format!("failed to parse `{}`: {source}", path.display()),
    })
}

/// Materialize `{consumer}/target/factorio-rs/bindings/{mod}/` from a manifest.
///
/// Skips rewriting when an existing stub's embedded content hash matches.
///
/// # Errors
/// Returns I/O errors when creating or writing stub files.
pub fn materialize_binding_crate(
    consumer_root: &Path,
    manifest: &ExportsManifest,
) -> CliResult<PathBuf> {
    let root = binding_crate_dir(consumer_root, &manifest.mod_name);
    let src = root.join("src");
    std::fs::create_dir_all(&src).map_err(|source| CliError::CreateDir {
        path: src.clone(),
        source,
    })?;

    let crate_name = manifest.mod_name.replace('_', "-");
    let remote_fn_names: BTreeSet<&str> = manifest
        .remotes
        .iter()
        .map(|remote| remote.function.as_str())
        .collect();
    let remote_fns_toml = if remote_fn_names.is_empty() {
        String::new()
    } else {
        let quoted: Vec<String> = remote_fn_names
            .iter()
            .map(|name| format!("\"{name}\""))
            .collect();
        format!("remote_fns = [{}]\n", quoted.join(", "))
    };

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "{version}"
edition = "2024"
publish = false
description = "Auto-generated factorio-rs API stubs for `{mod_name}`"

[lib]
path = "src/lib.rs"

[package.metadata.factorio]
mod_name = "{mod_name}"
dependencies = ["{dependency}"]
module_root = "{module_root}"
interface = "{interface}"
{remote_fns_toml}"#,
        version = manifest.version,
        mod_name = manifest.mod_name,
        dependency = manifest.dependency,
        module_root = manifest.module_root,
        interface = manifest.interface,
    );

    let lib_rs = generate_lib_rs_from_manifest(manifest);
    let stamp = content_stamp(&cargo_toml, &lib_rs);
    let stamp_path = root.join(".factorio-rs-stamp");
    if stamp_path
        .exists()
        .then(|| std::fs::read_to_string(&stamp_path).ok())
        .flatten()
        .is_some_and(|existing| existing.trim() == stamp)
    {
        return Ok(root);
    }

    std::fs::write(root.join("Cargo.toml"), cargo_toml).map_err(|source| CliError::WriteFile {
        path: root.join("Cargo.toml"),
        source,
    })?;
    std::fs::write(src.join("lib.rs"), lib_rs).map_err(|source| CliError::WriteFile {
        path: src.join("lib.rs"),
        source,
    })?;
    std::fs::write(&stamp_path, format!("{stamp}\n")).map_err(|source| CliError::WriteFile {
        path: stamp_path,
        source,
    })?;

    Ok(root)
}

/// Path to the ephemeral binding crate for `mod_name`.
#[must_use]
pub fn binding_crate_dir(consumer_root: &Path, mod_name: &str) -> PathBuf {
    consumer_root
        .join(BINDINGS_DIR_REL)
        .join(mod_name.replace('_', "-"))
}

/// Relative Cargo path string for the binding crate (`target/factorio-rs/bindings/provider`).
#[must_use]
pub fn binding_crate_rel_path(mod_name: &str) -> PathBuf {
    PathBuf::from(BINDINGS_DIR_REL).join(mod_name.replace('_', "-"))
}

/// Register / update a library path in the consumer's `.factorio-rs/libraries.json`.
///
/// # Errors
/// Returns I/O errors.
pub fn register_library(consumer_root: &Path, mod_name: &str, lib_rel_path: &str) -> CliResult<()> {
    let path = consumer_root.join(LIBRARIES_REGISTRY_REL);
    let mut registry = load_libraries_registry(consumer_root).unwrap_or_default();
    registry
        .libraries
        .insert(mod_name.to_string(), lib_rel_path.to_string());
    write_libraries_registry(consumer_root, &registry)?;
    let _ = path;
    Ok(())
}

/// Refresh all registered library stubs from their provider manifests.
///
/// # Errors
/// Propagates missing manifests / I/O errors for registered libraries.
pub fn refresh_registered_bindings(consumer_root: &Path) -> CliResult<Vec<PathBuf>> {
    let registry = load_libraries_registry(consumer_root).unwrap_or_default();
    let mut refreshed = Vec::new();
    for (mod_name, lib_rel) in &registry.libraries {
        let lib_root = consumer_root.join(lib_rel);
        let manifest = load_exports_manifest(&lib_root)?;
        if manifest.mod_name != *mod_name {
            return Err(CliError::CargoMetadata {
                message: format!(
                    "library at `{}` exports mod `{}`, expected `{mod_name}`",
                    lib_root.display(),
                    manifest.mod_name
                ),
            });
        }
        refreshed.push(materialize_binding_crate(consumer_root, &manifest)?);
    }
    Ok(refreshed)
}

fn load_libraries_registry(consumer_root: &Path) -> CliResult<LibrariesRegistry> {
    let path = consumer_root.join(LIBRARIES_REGISTRY_REL);
    if !path.exists() {
        return Ok(LibrariesRegistry::default());
    }
    let contents = std::fs::read_to_string(&path).map_err(|source| CliError::ReadFile {
        path: path.clone(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| CliError::CargoMetadata {
        message: format!("failed to parse `{}`: {source}", path.display()),
    })
}

fn write_libraries_registry(consumer_root: &Path, registry: &LibrariesRegistry) -> CliResult<()> {
    let path = consumer_root.join(LIBRARIES_REGISTRY_REL);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| CliError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let json =
        serde_json::to_string_pretty(registry).map_err(|source| CliError::CargoMetadata {
            message: format!("failed to serialize libraries registry: {source}"),
        })?;
    std::fs::write(&path, format!("{json}\n"))
        .map_err(|source| CliError::WriteFile { path, source })
}

fn build_manifest(
    package: &CargoPackage,
    remote_exports: &[RemoteExport],
    shared_exports: &[SharedExport],
    shared_consts: &[SharedConst],
) -> ExportsManifest {
    let interface = remote_exports
        .first()
        .map(|export| export.interface.clone())
        .unwrap_or_else(|| package.name.clone());

    ExportsManifest {
        mod_name: package.name.clone(),
        version: package.version.clone(),
        dependency: format!("{} >= {}", package.name, package.version),
        module_root: "lua".to_string(),
        interface,
        remotes: remote_exports
            .iter()
            .map(|export| ManifestRemote {
                function: export.function.clone(),
                interface: export.interface.clone(),
                params: export
                    .params
                    .iter()
                    .map(|(name, ty)| ManifestParam {
                        name: name.clone(),
                        ty: ty.clone(),
                    })
                    .collect(),
            })
            .collect(),
        shared_fns: shared_exports
            .iter()
            .map(|export| ManifestSharedFn {
                module: export.module.clone(),
                function: export.function.clone(),
                params: export
                    .params
                    .iter()
                    .map(|(name, ty)| ManifestParam {
                        name: name.clone(),
                        ty: ty.clone(),
                    })
                    .collect(),
            })
            .collect(),
        shared_consts: shared_consts
            .iter()
            .map(|konst| ManifestSharedConst {
                module: konst.module.clone(),
                name: konst.name.clone(),
                source_type: konst.source_type.clone(),
            })
            .collect(),
    }
}

fn generate_lib_rs_from_manifest(manifest: &ExportsManifest) -> String {
    let remotes: Vec<RemoteExport> = manifest
        .remotes
        .iter()
        .map(|remote| RemoteExport {
            module: String::new(),
            function: remote.function.clone(),
            interface: remote.interface.clone(),
            params: remote
                .params
                .iter()
                .map(|param| (param.name.clone(), param.ty.clone()))
                .collect(),
        })
        .collect();
    let shared: Vec<SharedExport> = manifest
        .shared_fns
        .iter()
        .map(|func| SharedExport {
            module: func.module.clone(),
            function: func.function.clone(),
            params: func
                .params
                .iter()
                .map(|param| (param.name.clone(), param.ty.clone()))
                .collect(),
        })
        .collect();
    let consts: Vec<SharedConst> = manifest
        .shared_consts
        .iter()
        .map(|konst| SharedConst {
            module: konst.module.clone(),
            name: konst.name.clone(),
            source_type: konst.source_type.clone(),
        })
        .collect();
    generate_lib_rs(&remotes, &shared, &consts)
}

fn generate_lib_rs(
    remote_exports: &[RemoteExport],
    shared_exports: &[SharedExport],
    shared_consts: &[SharedConst],
) -> String {
    let mut out = String::from(
        "//! Auto-generated by factorio-rs. Do not edit by hand.\n//!\n\
         //! Control-stage exports are at the crate root and lower to `remote.call`.\n\
         //! Shared exports mirror `__mod__/lua/...` require paths.\n\
         //! `pub mod remote` re-exports root remotes for compatibility.\n\n",
    );

    if !remote_exports.is_empty() {
        let mut seen = BTreeSet::new();
        let mut names = Vec::new();
        for export in remote_exports {
            if !seen.insert(export.function.clone()) {
                continue;
            }
            write_fn_stub_params(&mut out, 0, &export.function, &export.params);
            names.push(export.function.clone());
        }
        out.push('\n');
        out.push_str("pub mod remote {\n");
        for name in &names {
            let _ = writeln!(out, "    pub use super::{name};");
        }
        out.push_str("}\n\n");
    }

    write_shared_modules(&mut out, shared_exports, shared_consts);
    out
}

fn write_shared_modules(
    out: &mut String,
    shared_exports: &[SharedExport],
    shared_consts: &[SharedConst],
) {
    if shared_exports.is_empty() && shared_consts.is_empty() {
        return;
    }

    let mut tree = ModuleTree::default();
    for export in shared_exports {
        tree.insert_fn(&export.module, export.clone());
    }
    for konst in shared_consts {
        tree.insert_const(&konst.module, konst.clone());
    }
    tree.write(out, 0);
}

#[derive(Default)]
struct ModuleTree {
    functions: Vec<SharedExport>,
    consts: Vec<SharedConst>,
    children: BTreeMap<String, ModuleTree>,
}

impl ModuleTree {
    fn insert_fn(&mut self, module: &str, export: SharedExport) {
        let mut parts = module.split('.').filter(|part| !part.is_empty());
        let Some(first) = parts.next() else {
            self.functions.push(export);
            return;
        };
        let rest: Vec<&str> = parts.collect();
        let child = self.children.entry(first.to_string()).or_default();
        if rest.is_empty() {
            child.functions.push(export);
        } else {
            child.insert_fn(&rest.join("."), export);
        }
    }

    fn insert_const(&mut self, module: &str, konst: SharedConst) {
        let mut parts = module.split('.').filter(|part| !part.is_empty());
        let Some(first) = parts.next() else {
            self.consts.push(konst);
            return;
        };
        let rest: Vec<&str> = parts.collect();
        let child = self.children.entry(first.to_string()).or_default();
        if rest.is_empty() {
            child.consts.push(konst);
        } else {
            child.insert_const(&rest.join("."), konst);
        }
    }

    fn write(&self, out: &mut String, indent: usize) {
        self.write_children_only(out, indent);
    }

    fn write_children_only(&self, out: &mut String, indent: usize) {
        let pad = "    ".repeat(indent);
        for (name, child) in &self.children {
            let _ = writeln!(out, "{pad}pub mod {name} {{");
            for konst in &child.consts {
                let ty = konst.source_type.as_deref().unwrap_or("i32");
                let _ = writeln!(out, "{}    pub const {}: {ty} = 0;", pad, konst.name);
            }
            for func in &child.functions {
                write_fn_stub_params(out, indent + 1, &func.function, &func.params);
            }
            child.write_children_only(out, indent + 1);
            let _ = writeln!(out, "{pad}}}");
        }
    }
}

fn write_fn_stub_params(
    out: &mut String,
    indent: usize,
    name: &str,
    params: &[(String, Option<String>)],
) {
    let pad = "    ".repeat(indent);
    let params = params
        .iter()
        .map(|(param_name, ty)| {
            let ty = ty.as_deref().unwrap_or("()");
            format!("{param_name}: {ty}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(out, "{pad}#[allow(unused_variables)]");
    let _ = writeln!(out, "{pad}pub fn {name}({params}) {{}}");
}

fn content_stamp(cargo_toml: &str, lib_rs: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    cargo_toml.hash(&mut hasher);
    lib_rs.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn write_and_materialize_round_trip() {
        let temp = tempfile::TempDir::new().unwrap();
        let provider = temp.path().join("provider");
        let consumer = temp.path().join("consumer");
        std::fs::create_dir_all(&provider).unwrap();
        std::fs::create_dir_all(&consumer).unwrap();

        let package = CargoPackage {
            name: "provider".to_string(),
            version: "0.1.3".to_string(),
            authors: None,
        };
        let remotes = vec![RemoteExport {
            module: "control".to_string(),
            function: "add".to_string(),
            interface: "provider".to_string(),
            params: vec![
                ("a".to_string(), Some("i32".to_string())),
                ("b".to_string(), Some("i32".to_string())),
            ],
        }];
        let path = write_exports_manifest(&provider, &package, &remotes, &[], &[])
            .unwrap()
            .unwrap();
        assert!(path.exists());

        let manifest = load_exports_manifest(&provider).unwrap();
        let stub = materialize_binding_crate(&consumer, &manifest).unwrap();
        let lib = std::fs::read_to_string(stub.join("src/lib.rs")).unwrap();
        assert!(lib.contains("pub fn add(a: i32, b: i32)"));
        let cargo = std::fs::read_to_string(stub.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"provider\""));
        assert!(cargo.contains("remote_fns = [\"add\"]"));

        // Second materialize is a stamp no-op (still ok).
        materialize_binding_crate(&consumer, &manifest).unwrap();
    }
}
