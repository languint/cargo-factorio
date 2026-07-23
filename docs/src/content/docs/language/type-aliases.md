---
title: Type aliases
description: Transparent type Name = ... aliases for Option, Vec, and nested names.
---

`type` aliases are **transparent**: the frontend substitutes the aliased type
(including nested and generic aliases), then emits **no Lua** for the alias.

```rust
type Count = i64;
type Entities = Vec<i64>;
type Opt<T> = Option<T>;

fn use_aliases(n: Count, values: Entities, maybe: Opt<i64>) {
    let total: Count = n;
    for value in values {
        let _ = value;
    }
    if let Some(x) = maybe {
        let _ = x;
    }
}
```

After resolution, `Entities` still counts as `Vec` (ordered `ipairs`), and
`Opt<_>` still counts as `Option` for `if let Some` / lints.

## Supported forms

| OK | Rejected |
| --- | --- |
| `type Name = Path` | Lifetimes (`type A<'a> = ...`) |
| `type Name<T> = ...` | Const params |
| Nested aliases (`type A = B`) | Bounds / `where` clauses |
| Block-local `type` inside a function | |

## See also

- [Filter entity lists](/recipes/filter-entities/) - aliases + iterators
- [Option and Result](/guides/option-and-result/)
