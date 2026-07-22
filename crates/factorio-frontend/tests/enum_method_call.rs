#![allow(clippy::expect_used, clippy::unwrap_used)]
mod common;

use common::must_ok_parse;
use factorio_codegen::LuaGenerator;
use factorio_frontend::parse_module;

#[test]
fn enum_method_call_and_matches_codegen() {
    let _phase = must_ok_parse(parse_module(
        r"
        pub enum Phase {
            Idle,
            Running { ticks: i64 },
        }
        impl Phase {
            pub fn tick(self) -> Phase {
                match self {
                    Phase::Idle => Phase::Running { ticks: 0 },
                    Phase::Running { ticks } => Phase::Running { ticks: ticks + 1 },
                }
            }
        }
        ",
        "shared.phase",
    ));
    let boot = must_ok_parse(parse_module(
        r"
        use crate::shared::phase::Phase;
        pub fn go() {
            let mut phase = Phase::Idle;
            phase = phase.tick();
            let running = matches!(phase, Phase::Running { .. });
            let _ = running;
        }
        ",
        "control.boot",
    ));
    let lua = LuaGenerator::with_mod_name("playground")
        .generate_module(&boot)
        .expect("lua");
    assert!(
        lua.contains("Phase.tick(phase)") || lua.contains("phase:tick()"),
        "{lua}"
    );
    assert!(lua.contains("tag == \"Running\""), "{lua}");
}
