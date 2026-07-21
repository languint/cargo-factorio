use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    process::Command,
};

use factorio_frontend::{BindingRegistry, FactorioBinding};
use serde::Deserialize;

use crate::error::{CliError, CliResult};

#[derive(Debug, Deserialize)]
struct FactorioPackageMetadata {
    mod_name: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default = "default_module_root")]
    module_root: String,
    #[serde(default)]
    interface: Option<String>,
    #[serde(default)]
    remote_fns: Vec<String>,
    /// Shared `#[factorio_rs::inline]` symbols (require hot path; never remote).
    #[serde(default)]
    inline_fns: Vec<String>,
}

fn default_module_root() -> String {
    "lua".to_string()
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    resolve: Option<MetadataResolve>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    manifest_path: String,
    targets: Vec<MetadataTarget>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct MetadataTarget {
    name: String,
    kind: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataResolve {
    root: Option<String>,
    nodes: Vec<ResolveNode>,
}

#[derive(Debug, Deserialize)]
struct ResolveNode {
    id: String,
    dependencies: Vec<String>,
}

/// Discover direct Cargo dependencies that publish `[package.metadata.factorio]`.
///
/// # Errors
/// Returns [`CliError::CargoMetadata`] if `cargo metadata` fails or its JSON /
/// `[package.metadata.factorio]` cannot be parsed.
pub fn discover_bindings(project_root: &Path) -> CliResult<BindingRegistry> {
    let manifest_path = project_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(&manifest_path)
        .output()
        .map_err(|source| CliError::CargoMetadata {
            message: format!("failed to run `cargo metadata`: {source}"),
        })?;

    if !output.status.success() {
        return Err(CliError::CargoMetadata {
            message: format!(
                "`cargo metadata` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).map_err(|source| CliError::CargoMetadata {
            message: format!("failed to parse `cargo metadata` JSON: {source}"),
        })?;

    let Some(resolve) = metadata.resolve.as_ref() else {
        return Ok(BTreeMap::new());
    };

    let manifest_canon =
        std::fs::canonicalize(&manifest_path).unwrap_or_else(|_| manifest_path.clone());
    let root_package = metadata.packages.iter().find(|package| {
        let package_manifest = Path::new(&package.manifest_path);
        package_manifest == manifest_path
            || std::fs::canonicalize(package_manifest).is_ok_and(|path| path == manifest_canon)
    });

    let root_id = root_package
        .map(|package| package.id.as_str())
        .or(resolve.root.as_deref());

    let Some(root_id) = root_id else {
        return Ok(BTreeMap::new());
    };

    let Some(root_node) = resolve.nodes.iter().find(|node| node.id == root_id) else {
        return Ok(BTreeMap::new());
    };

    let packages_by_id: BTreeMap<&str, &MetadataPackage> = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect();

    let mut bindings = BTreeMap::new();
    for dep_id in &root_node.dependencies {
        let Some(package) = packages_by_id.get(dep_id.as_str()) else {
            continue;
        };
        let Some(factorio) = parse_factorio_metadata(package.metadata.as_ref())? else {
            continue;
        };

        let crate_name = rust_crate_name(package);
        let remote_fns: BTreeSet<String> = factorio.remote_fns.into_iter().collect();
        let inline_fns: BTreeSet<String> = factorio.inline_fns.into_iter().collect();
        bindings.insert(
            crate_name.clone(),
            FactorioBinding {
                crate_name,
                mod_name: factorio.mod_name,
                dependencies: factorio.dependencies,
                module_root: factorio.module_root,
                interface: factorio.interface,
                remote_fns,
                inline_fns,
            },
        );
    }

    Ok(bindings)
}

fn rust_crate_name(package: &MetadataPackage) -> String {
    package
        .targets
        .iter()
        .find(|target| {
            target
                .kind
                .iter()
                .any(|kind| kind == "lib" || kind == "rlib" || kind == "dylib" || kind == "cdylib")
        })
        .map_or_else(
            || package.name.replace('-', "_"),
            |target| target.name.clone(),
        )
}

fn parse_factorio_metadata(
    metadata: Option<&serde_json::Value>,
) -> CliResult<Option<FactorioPackageMetadata>> {
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let Some(factorio) = metadata.get("factorio") else {
        return Ok(None);
    };
    if factorio.is_null() {
        return Ok(None);
    }
    let parsed: FactorioPackageMetadata =
        serde_json::from_value(factorio.clone()).map_err(|source| CliError::CargoMetadata {
            message: format!("invalid `[package.metadata.factorio]`: {source}"),
        })?;
    Ok(Some(parsed))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn parses_factorio_metadata_defaults_module_root() {
        let value = serde_json::json!({
            "factorio": {
                "mod_name": "provider",
                "dependencies": ["provider >= 0.1.0"]
            }
        });
        let parsed = parse_factorio_metadata(Some(&value)).unwrap().unwrap();
        assert_eq!(parsed.mod_name, "provider");
        assert_eq!(parsed.module_root, "lua");
        assert_eq!(parsed.dependencies, vec!["provider >= 0.1.0"]);
    }

    #[test]
    fn parses_empty_module_root() {
        let value = serde_json::json!({
            "factorio": {
                "mod_name": "flib",
                "module_root": "",
                "dependencies": ["flib >= 0.14"]
            }
        });
        let parsed = parse_factorio_metadata(Some(&value)).unwrap().unwrap();
        assert_eq!(parsed.module_root, "");
    }

    #[test]
    fn parses_remote_fns() {
        let value = serde_json::json!({
            "factorio": {
                "mod_name": "provider",
                "interface": "provider",
                "remote_fns": ["greet", "ping"]
            }
        });
        let parsed = parse_factorio_metadata(Some(&value)).unwrap().unwrap();
        assert_eq!(parsed.remote_fns, vec!["greet", "ping"]);
        assert_eq!(parsed.interface.as_deref(), Some("provider"));
    }
}
