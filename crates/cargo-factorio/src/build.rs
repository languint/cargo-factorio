use std::path::{Path, PathBuf};

use factorio_codegen::LuaGenerator;
use factorio_frontend::parse_module;

use crate::{
    config::Config,
    error::{CliError, CliResult},
};

/// Transpile Rust sources in a project to Lua.
pub fn build(project_root: &Path) -> CliResult<Vec<PathBuf>> {
    let config = Config::load(project_root)?;
    let source_dir = project_root.join(&config.source);
    let output_dir = project_root.join(&config.output_dir);

    if !source_dir.is_dir() {
        return Err(CliError::NotFound {
            path: source_dir,
        });
    }

    std::fs::create_dir_all(&output_dir).map_err(|source| CliError::CreateDir {
        path: output_dir.clone(),
        source,
    })?;

    let mut outputs = Vec::new();
    let mut found_source = false;

    for entry in std::fs::read_dir(&source_dir).map_err(|source| CliError::ReadDir {
        path: source_dir.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| CliError::ReadDir {
            path: source_dir.clone(),
            source,
        })?;
        let path = entry.path();

        if !is_transpilable_source(&path) {
            continue;
        }

        found_source = true;
        outputs.push(transpile_file(&path, &output_dir)?);
    }

    if !found_source {
        return Err(CliError::NoSourceFiles { path: source_dir });
    }

    Ok(outputs)
}

fn is_transpilable_source(path: &Path) -> bool {
    is_rust_source(path)
        && path
            .file_name()
            .is_some_and(|file_name| file_name != "lib.rs")
}

fn is_rust_source(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
}

fn transpile_file(source_path: &Path, output_dir: &Path) -> CliResult<PathBuf> {
    let module_name = source_path
        .file_stem()
        .ok_or_else(|| CliError::InvalidProjectPath {
            path: source_path.to_path_buf(),
        })?
        .to_string_lossy()
        .into_owned();

    let source = std::fs::read_to_string(source_path).map_err(|err| CliError::ReadFile {
        path: source_path.to_path_buf(),
        source: err,
    })?;

    let module = parse_module(&source, &module_name)?;
    let lua = LuaGenerator::new().generate_module(&module)?;

    let output_path = output_dir.join(format!("{module_name}.lua"));
    std::fs::write(&output_path, lua).map_err(|source| CliError::WriteFile {
        path: output_path.clone(),
        source,
    })?;

    Ok(output_path)
}
