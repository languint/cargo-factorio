use crate::{
    error::{FrontendError, FrontendResult},
    paths::{require_local_name, split_crate_path},
};

use super::imports::ImportFragment;

pub struct LowerContext<'a> {
    pub imports: &'a mut Vec<ImportFragment>,
}

impl LowerContext<'_> {
    fn register_crate_module(&mut self, module: &str) {
        if self
            .imports
            .iter()
            .any(|fragment| fragment.module == module)
        {
            return;
        }

        self.imports.push(ImportFragment {
            module: module.to_string(),
            require_local: require_local_name(module),
            item: None,
        });
    }

    pub fn normalize_crate_path(&mut self, segments: &mut Vec<String>) -> FrontendResult<()> {
        if segments.first().map(String::as_str) != Some("crate") {
            return Ok(());
        }

        segments.remove(0);
        if segments.is_empty() {
            return Err(FrontendError::UnsupportedExpression {
                location: "crate".to_string(),
            });
        }

        let (module_path, rest) = split_crate_path(segments);
        if module_path.is_empty() {
            return Err(FrontendError::UnsupportedExpression {
                location: segments.join("::"),
            });
        }

        self.register_crate_module(&module_path);

        let local = require_local_name(&module_path);
        *segments = if rest.is_empty() {
            vec![local]
        } else {
            let mut rewritten = vec![local];
            rewritten.extend(rest);
            rewritten
        };

        Ok(())
    }
}
