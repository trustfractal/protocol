#! /bin/bash

set -euxo pipefail

CARGO_INCREMENTAL=false cargo build --jobs 2 --release --manifest-path explorer/Cargo.toml

rm target/release/fractal_explorer_*
mv explorer/target/release/fractal_explorer_* target/release/

POSTGRES="${1:-"postgres://shelby:shelby@localhost/fractal_protocol_explorer"}"

PORT="${PORT:-8080}" \
  HEROKU_POSTGRESQL_AQUA_URL=$POSTGRES \
  foreman start -f explorer/Procfile -d . web
