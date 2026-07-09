use proc_macro2::{Ident, Span};

const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];

pub fn sanitize_ident(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub fn make_ident(name: &str) -> Ident {
    let sanitized = sanitize_ident(name);
    if RUST_KEYWORDS.contains(&sanitized.as_str()) {
        Ident::new_raw(&sanitized, Span::call_site())
    } else {
        Ident::new(&sanitized, Span::call_site())
    }
}
