#!/usr/bin/env sh
set -eu
cargo test --workspace
cargo run -p csxc -- check examples/expressions/Program.csx
cargo run -p csxc -- build examples/expressions/Program.csx
./csx-out/csx-app
