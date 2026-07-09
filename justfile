alias b := build
alias t := test

[private]
default:
    @just --list

build *ARGS="--workspace --all-targets":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f Cargo.toml ]; then
      cd {{ invocation_directory() }}
    fi
    cargo build {{ ARGS }}

test: build
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f Cargo.toml ]; then
      cd {{ invocation_directory() }}
    fi
    cargo nextest run

update-factorio-api:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f Cargo.toml ]; then
      cd {{ invocation_directory() }}
    fi
    curl -sL "https://lua-api.factorio.com/latest/runtime-api.json" \
      -o crates/factorio-api-gen/api/runtime-api.json
    cargo build -p factorio-api -p factorio-macros

format:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f Cargo.toml ]; then
      cd {{ invocation_directory() }}
    fi
    cargo fmt

clippy *ARGS="--locked --offline --workspace --all-targets":
    cargo clippy {{ ARGS }} -- --deny warnings --allow deprecated
