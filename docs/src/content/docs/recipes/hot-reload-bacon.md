---
title: Hot reload with Bacon
description: Watch Rust sources with Bacon; sync into Factorio for control-stage reload and re-run in-game tests without restarting.
---

Use **[Bacon](https://dystroy.org/bacon/)** as the file watcher. factorio-rs exposes jobs Bacon can run - there is no custom `factorio-rs watch` (open an issue if you would like one).

## Install Bacon

```bash
cargo install --locked bacon
```

## Scaffold with jobs

```bash
factorio-rs init --name my-mod --bacon
```

Or copy this `bacon.toml` into an existing project:

```toml
default_job = "factorio-reload"
env.CARGO_TERM_COLOR = "always"
# Prefer `dist` over `dist/**` (directory-level notify events).
ignore = ["dist", ".factorio-rs", "target", "src/factorio_exports.rs"]

[jobs.factorio-check]
command = ["factorio-rs", "check"]
need_stdout = true
# Important in Cargo workspaces: default watches include `examples/` recursively.
default_watch = false
watch = ["src", "Factorio.toml", "Cargo.toml"]

[jobs.factorio-reload]
command = ["factorio-rs", "sync", "--symlink", "--hot-reload"]
need_stdout = true
default_watch = false
watch = ["src", "Factorio.toml", "Cargo.toml"]
show_command_error_code = true

[jobs.factorio-test]
command = ["factorio-rs", "test", "--rerun"]
need_stdout = true
default_watch = false
watch = ["src", "Factorio.toml", "Cargo.toml"]
show_command_error_code = true

[keybindings]
c = "job:factorio-check"
r = "job:factorio-reload"
t = "job:factorio-test"
```

`CARGO_TERM_COLOR=always` keeps status and test colors when Bacon pipes output.
Failure reports use cargo’s `---- name stdout ----` layout so Bacon shows the
assertion message instead of italic “no output”.

Hot-reload generation tracks **source** changes (`src/**/*.rs`, `Factorio.toml`,
`Cargo.toml`), not `dist/` rewrites. Identical sources print
`generation N (unchanged)` and skip the UDP ping.

If Bacon still loops after a sync, check that jobs set `default_watch = false`
(Bacon’s defaults watch workspace `examples/`, which includes each example’s
`dist/`).

## In-game control reload

Factorio keeps mod Lua in an in-memory VFS for the whole session, so the CLI cannot detect on-disk edits by re-`require`ing a generation file. Instead:

1. Launch Factorio with localhost UDP enabled (port **34201** by default,
   override with `FACTORIO_RS_UDP_PORT`):
   - `factorio-rs open` / `factorio-rs install --open` pass `--enable-lua-udp=34201`
   - Steam: set launch options to `--enable-lua-udp=34201`
2. Install once and load a save: `factorio-rs sync --symlink --hot-reload`
3. In another terminal: `bacon` (default job is `factorio-reload`) or
   `bacon -j factorio-reload`
4. Edit control-stage Rust -> Bacon syncs -> CLI sends a UDP ping -> the in-game
   probe calls `game.reload_mods()` (then a second pass automatically).

You should see the world briefly reload when the ping lands (no chat spam).

`sync --hot-reload` also writes `lua/factorio_rs_reload_gen.lua` after deploy
(for tooling) and injects the UDP probe on `control.lua`. Prefer `--symlink`
so the mods entry points at `dist/` (Unix; falls back to copy).

**Data / settings stage:** prototype and settings changes still need a full
Factorio restart. `sync` prints a note when those stage files change.

## Automated tests without restarting Factorio

```bash
bacon -j factorio-test
```

`factorio-rs test --rerun`:

1. Builds the suite with a listen-capable harness and hot-reload probe.
2. Starts a headless Factorio listen process if none is running.
3. On later triggers, bumps reload generation, waits for the next `suite_end`, prints the report, and exits (Bacon-friendly).

Manual equivalents:

```bash
factorio-rs test --listen    # keep Factorio alive after the first suite
factorio-rs test --rerun     # rebuild + wait for the next suite
```

See [Testing](/guides/testing/) for the suite protocol and `--gui`.

## See also

- [CLI reference](/reference/cli/) - `sync`, `test --listen` / `--rerun`
- [First hour](/recipes/first-hour/) - install loop without Bacon
- [Stages](/guides/stages/) - control vs data vs settings
