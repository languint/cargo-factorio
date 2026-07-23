use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    LitStr, Path, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

pub fn locale(input: TokenStream) -> TokenStream {
    let LocaleInput { languages, .. } = parse_macro_input!(input as LocaleInput);

    let mut checks = Vec::<TokenStream2>::new();
    for lang in &languages {
        for category in &lang.categories {
            for entry in &category.entries {
                if let LocaleKey::Path(path) = &entry.key {
                    checks.push(quote::quote! {
                        let _: &'static str = #path;
                    });
                }
                let value = &entry.value;
                let value_text = value.value();
                if value_text.contains('\n') || value_text.contains('\r') {
                    return syn::Error::new_spanned(value, "locale values must be a single line")
                        .to_compile_error()
                        .into();
                }
            }
        }
    }

    TokenStream::from(quote::quote! {
        const _: () = {
            #( #checks )*
        };
    })
}

struct LocaleInput {
    #[allow(dead_code)]
    file: Option<String>,
    languages: Vec<LocaleLanguageBlock>,
}

struct LocaleLanguageBlock {
    #[allow(dead_code)]
    lang: String,
    categories: Vec<LocaleCategoryBlock>,
}

struct LocaleCategoryBlock {
    #[allow(dead_code)]
    name: String,
    entries: Vec<LocaleEntry>,
}

struct LocaleEntry {
    key: LocaleKey,
    value: LitStr,
}

enum LocaleKey {
    Path(Path),
    #[allow(dead_code)]
    Literal(String),
}

impl Parse for LocaleInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut file = None;
        if input.peek(syn::Ident) {
            let fork = input.fork();
            let kw: syn::Ident = fork.parse()?;
            if kw == "file" && fork.peek(Token![=]) {
                let _: syn::Ident = input.parse()?;
                let _: Token![=] = input.parse()?;
                let lit: LitStr = input.parse()?;
                file = Some(lit.value());
                let _: Option<Token![,]> = input.parse()?;
            }
        }

        let mut languages = Vec::new();
        while !input.is_empty() {
            languages.push(input.parse()?);
        }

        if languages.is_empty() {
            return Err(input.error("expected at least one language block such as `en { ... }`"));
        }

        Ok(Self { file, languages })
    }
}

impl Parse for LocaleLanguageBlock {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lang = if input.peek(LitStr) {
            input.parse::<LitStr>()?.value()
        } else {
            input.parse::<syn::Ident>()?.to_string()
        };

        let content;
        syn::braced!(content in input);

        let mut categories = Vec::new();
        while !content.is_empty() {
            categories.push(content.parse()?);
        }

        Ok(Self { lang, categories })
    }
}

impl Parse for LocaleCategoryBlock {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = if input.peek(LitStr) {
            input.parse::<LitStr>()?.value()
        } else {
            let ident: syn::Ident = input.parse()?;
            ident.to_string().replace('_', "-")
        };

        let content;
        syn::braced!(content in input);

        let mut entries = Vec::new();
        while !content.is_empty() {
            entries.push(content.parse()?);
        }

        Ok(Self { name, entries })
    }
}

impl Parse for LocaleEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let key = if input.peek(LitStr) {
            LocaleKey::Literal(input.parse::<LitStr>()?.value())
        } else {
            LocaleKey::Path(input.parse()?)
        };
        let _: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;
        let _: Option<Token![,]> = input.parse()?;
        Ok(Self { key, value })
    }
}
