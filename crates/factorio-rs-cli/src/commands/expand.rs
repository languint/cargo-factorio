//! Expand macros with rustc before frontend lowering.
//!
//! Uses `-Zunpretty=expanded` (nightly-only flag). On stable toolchains we set
//! `RUSTC_BOOTSTRAP=1` so the same rustc that typechecked the crate can dump
//! the fully expanded AST - including `macro_rules!` and dependency proc macros.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::error::{CliError, CliResult};

/// Expand the package library crate to a single Rust source string.
///
/// # Errors
/// Returns [`CliError::MacroExpandFailed`] when rustc/cargo fail or produce empty output.
pub fn expand_crate(project_root: &Path) -> CliResult<String> {
    let manifest = project_root.join("Cargo.toml");
    let mut command = Command::new("cargo");
    command
        .arg("rustc")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("--profile=check")
        .arg("--lib")
        .arg("--")
        .arg("-Zunpretty=expanded")
        .env("RUSTC_BOOTSTRAP", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command.output().map_err(|source| CliError::CargoMetadata {
        message: format!("failed to run macro expansion (`cargo rustc`): {source}"),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliError::MacroExpandFailed {
            message: truncate_for_error(&stderr),
        });
    }

    let expanded = String::from_utf8(output.stdout).map_err(|source| CliError::CargoMetadata {
        message: format!("macro expansion output was not UTF-8: {source}"),
    })?;

    if expanded.trim().is_empty() {
        return Err(CliError::MacroExpandFailed {
            message: "rustc produced empty expansion output".to_string(),
        });
    }

    Ok(expanded)
}

fn truncate_for_error(stderr: &str) -> String {
    const MAX: usize = 4_000;
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        return "cargo rustc -Zunpretty=expanded failed (no stderr)".to_string();
    }
    if trimmed.len() <= MAX {
        return trimmed.to_string();
    }
    format!("{}…", &trimmed[..MAX])
}
