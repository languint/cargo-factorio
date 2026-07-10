use proc_macro2::{Ident, Span};

pub fn sanitize_doc(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '`' && chars.peek() == Some(&'`') {
            chars.next();
            if chars.peek() == Some(&'`') {
                chars.next();
                // let mut tag = String::new();
                // for ch in chars.by_ref() {
                //     if ch == '\n' {
                //         break;
                //     }
                //     tag.push(ch);
                // }
                result.push_str("```text\n");
                continue;
            }
            result.push_str("``");
        } else {
            result.push(c);
        }
    }
    result
}

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

pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
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
