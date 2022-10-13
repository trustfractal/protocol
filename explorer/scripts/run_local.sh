#! /bin/bash

set -euxo pipefail

CARGO_INCREMENTAL=false cargo test --jobs 2 --release --manifest-path explorer/Cargo.toml
CARGO_INCREMENTAL=false cargo build --jobs 2 --release --manifest-path explorer/Cargo.toml

rm target/release/fractal_explorer_*
mv explorer/target/release/fractal_explorer_* target/release/

POSTGRES="${1:-"postgres://juliosantos@localhost/fractal_protocol_explorer?sslmode=disable"}"

PORT="${PORT:-8080}" \
  HEROKU_POSTGRESQL_AQUA_URL=$POSTGRES \
  ACALA_BURNER_ADDRESS="0xE5067452913E5bDd9E726Dde21c66498594a8236" \
  ACALA_CHAIN_ID=686 \
  ACALA_FCL_TOKEN_ADDRESS="0xf1c1588Edf48EFd98c9822320cC869dA926657B7" \
  ACALA_URL="https://eth-rpc-karura.aca-api.network" \
  SUBSTRATE_CHAIN_URL="wss://main.devnet.fractalprotocol.com:443" \
  foreman start -f explorer/Procfile.dev -d . web
