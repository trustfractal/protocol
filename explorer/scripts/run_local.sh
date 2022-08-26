#! /bin/bash

set -euxo pipefail

cargo build --release --manifest-path explorer/Cargo.toml

POSTGRES="${1:-"postgres://shelby:shelby@localhost/fractal_protocol_explorer"}"

PORT="${PORT:-8080}" \
  HEROKU_POSTGRESQL_AQUA_URL=$POSTGRES \
  foreman start -f explorer/Procfile -d . web
