use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use factorio_ir::locale::LocaleFile;

use crate::error::{CliError, CliResult};

pub fn write_locale_files(output_dir: &Path, locales: &[LocaleFile]) -> CliResult<Vec<PathBuf>> {
    if locales.is_empty() {
        return Ok(Vec::new());
    }

    let mut merged: BTreeMap<(String, String), LocaleFile> = BTreeMap::new();
    for locale in locales {
        let key = (locale.lang.clone(), locale.file.clone());
        merged
            .entry(key)
            .and_modify(|existing| existing.entries.extend(locale.entries.iter().cloned()))
            .or_insert_with(|| locale.clone());
    }

    let mut written = Vec::new();
    for locale in merged.into_values() {
        let lang_dir = output_dir.join("locale").join(&locale.lang);
        std::fs::create_dir_all(&lang_dir).map_err(|source| CliError::CreateDir {
            path: lang_dir.clone(),
            source,
        })?;

        let path = lang_dir.join(format!("{}.cfg", locale.file));
        let contents = locale.to_cfg();
        std::fs::write(&path, contents).map_err(|source| CliError::WriteFile {
            path: path.clone(),
            source,
        })?;
        written.push(path);
    }

    Ok(written)
}
