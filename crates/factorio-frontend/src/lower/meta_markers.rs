//! Durable metadata markers emitted by `factorio-macros` that survive rustc expansion.

use std::collections::{HashMap, HashSet};

use syn::{Expr, Item, ItemConst, Lit};

use factorio_ir::function::ExportMeta;

use super::event_filter::lower_event_filter_list;

const STAGE_MARKER: &str = "__factorio_rs_stage";
const EXPORT_PREFIX: &str = "__factorio_rs_export__";
const INLINE_PREFIX: &str = "__factorio_rs_inline__";
const EVENT_PREFIX: &str = "__factorio_rs_event__";

/// Per-module metadata recovered from expansion-surviving consts.
#[derive(Debug, Default, Clone)]
pub struct ModuleMetaMarkers {
    pub stage: Option<String>,
    pub exports: HashMap<String, ExportMeta>,
    pub inlines: HashSet<String>,
    /// Function name -> optional lowered event filter.
    pub events: HashMap<String, Option<factorio_ir::expression::Expression>>,
}

/// Collect `__factorio_rs_*` marker consts from a module's items.
#[must_use]
pub fn collect_module_meta_markers(items: &[Item]) -> ModuleMetaMarkers {
    let mut meta = ModuleMetaMarkers::default();
    for item in items {
        let Item::Const(item_const) = item else {
            continue;
        };
        let name = item_const.ident.to_string();
        if name == STAGE_MARKER {
            if let Some(stage) = const_str_value(item_const) {
                meta.stage = Some(stage);
            }
            continue;
        }
        if let Some(fn_name) = name.strip_prefix(EXPORT_PREFIX) {
            let interface = const_str_value(item_const).filter(|value| !value.is_empty());
            meta.exports
                .insert(fn_name.to_string(), ExportMeta { interface });
            continue;
        }
        if let Some(fn_name) = name.strip_prefix(INLINE_PREFIX) {
            meta.inlines.insert(fn_name.to_string());
            meta.exports
                .entry(fn_name.to_string())
                .or_insert(ExportMeta { interface: None });
            continue;
        }
        if let Some(fn_name) = name.strip_prefix(EVENT_PREFIX) {
            let filter = const_str_value(item_const)
                .filter(|value| !value.is_empty())
                .and_then(|source| syn::parse_str::<Expr>(&source).ok())
                .and_then(|expr| lower_event_filter_list(&expr).ok());
            meta.events.insert(fn_name.to_string(), filter);
        }
    }
    meta
}

/// Whether this const is a factorio-rs expansion marker (skip IR emission).
#[must_use]
pub fn is_meta_marker_const(item: &ItemConst) -> bool {
    let name = item.ident.to_string();
    name == STAGE_MARKER
        || name.starts_with(EXPORT_PREFIX)
        || name.starts_with(INLINE_PREFIX)
        || name.starts_with(EVENT_PREFIX)
}

fn const_str_value(item: &ItemConst) -> Option<String> {
    match item.expr.as_ref() {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(lit) => Some(lit.value()),
            _ => None,
        },
        _ => None,
    }
}
