//! Lower `factorio_rs::test::steps()` to the harness intrinsic `__frs_steps()`.

use syn::Expr;

/// If `func` is `factorio_rs::test::steps`, return true.
pub fn is_factorio_rs_test_steps(func: &Expr) -> bool {
    let Expr::Path(path) = func else {
        return false;
    };
    let segments: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    match segments.as_slice() {
        [.., crate_name, module, name]
            if crate_name == "factorio_rs" && module == "test" && name == "steps" =>
        {
            true
        }
        // Bare `steps()` is not rewritten (too ambiguous). Callers should use the
        // full path so it lowers to `__frs_steps`.
        _ => false,
    }
}
