---
title: State machines with enums
description: Model mod phases and messages with user-defined enums and match.
---

User-defined enums lower to tagged Lua tables. Use them for phases, commands,
or any closed set of cases you want to `match` on - distinct from Factorio API
string unions like `GuiDirection`.

## Define the machine

```rust
enum Phase {
    Idle,
    Mining { ticks: i64 },
    Done,
}

impl Phase {
    fn tick(self) -> Phase {
        match self {
            Phase::Idle => Phase::Mining { ticks: 0 },
            Phase::Mining { ticks } if ticks + 1 >= 60 => Phase::Done,
            Phase::Mining { ticks } => Phase::Mining { ticks: ticks + 1 },
            Phase::Done => Phase::Done,
        }
    }
}
```

## What Lua looks like

| Rust | Lua shape |
| --- | --- |
| `Phase::Idle` | `{ tag = "Idle" }` |
| `Phase::Mining { ticks: 3 }` | `{ tag = "Mining", ticks = 3 }` |
| Tuple variant `Msg::Move(x, y)` | `{ tag = "Move", _1 = x, _2 = y }` |

Inherent methods share a table with unit variant constants, like structs.

## Drive it from an event

```rust
use factorio_rs::prelude::*;

// Keep the current phase in storage or a local you refresh each event.
fn on_tick(phase: Phase) -> Phase {
    let next = phase.tick();
    match &next {
        Phase::Done => println!("finished"),
        Phase::Mining { ticks } => println!("mining t={ticks}"),
        Phase::Idle => {}
    }
    next
}
```

Persist the phase if it must survive saves - [Persist with storage](persist-storage/).

## API string unions are different

```rust
// Factorio literal union -> plain Lua string
let dir = GuiDirection::Horizontal; // "horizontal"
```

Your `enum Phase` is a tagged table. Do not mix the two mental models.
Details: [Enums](../language/enums/).

## See also

- [Supported Rust](../guides/language/#match) - pattern rules
- [Option and Result](../guides/option-and-result/) - `Some` / `Ok` arms
