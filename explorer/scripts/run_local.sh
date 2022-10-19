#! /bin/bash

set -euxo pipefail

CARGO_INCREMENTAL=false cargo test --jobs 2 --release --manifest-path explorer/Cargo.toml
CARGO_INCREMENTAL=false cargo build --jobs 2 --release --manifest-path explorer/Cargo.toml

rm target/release/fractal_explorer_*
mv explorer/target/release/fractal_explorer_* target/release/

POSTGRES="${1:-"postgres://juliosantos@localhost/fractal_protocol_explorer?sslmode=disable"}"

PORT="${PORT:-8080}" \
  HEROKU_POSTGRESQL_AQUA_URL=$POSTGRES \
  ACALA_BURNER_ADDRESS="0xa952afE5c21d0D482E390F173F52d9c98383eDa0" \
  ACALA_CHAIN_ID=787 \
  ACALA_FCL_TOKEN_ADDRESS="0x477eBd116029877D108C9054be9d0Da01e85cd27" \
  ACALA_URL="https://eth-rpc-acala.aca-staging.network" \
  ACALA_STORAGE_BYTE_DEPOSIT=300000000000000 \
  SUBSTRATE_CHAIN_URL="wss://main.devnet.fractalprotocol.com:443" \
  foreman start -f explorer/Procfile.dev -d . web
