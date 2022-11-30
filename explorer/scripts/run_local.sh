#! /bin/bash

set -euxo pipefail

CARGO_INCREMENTAL=false cargo test --jobs 2 --release --manifest-path explorer/Cargo.toml
CARGO_INCREMENTAL=false cargo build --jobs 2 --release --manifest-path explorer/Cargo.toml

rm target/release/fractal_explorer_*
mv explorer/target/release/fractal_explorer_* target/release/

POSTGRES="${1:-"postgres://juliosantos@localhost/fractal_protocol_explorer?sslmode=disable"}"

PORT="${PORT:-8080}" \
  HEROKU_POSTGRESQL_AQUA_URL=$POSTGRES \
  GNOSIS_BURNER_ADDRESS="0x265B056E3Ec5fDC08FB79d37cc9a2551d1c1c231" \
  GNOSIS_CHAIN_ID=100 \
  GNOSIS_FCL_MINTER_KEY="" \
  GNOSIS_FCL_TOKEN_ADDRESS="0xb2B90d3C7A9EB291c4fA06cFc1EFE5AdDdCa7FD4" \
  GNOSIS_URL="https://rpc.gnosischain.com" \
  GNOSIS_EXPLORER_URL="https://blockscout.com/xdai/mainnet" \
  SUBSTRATE_CHAIN_URL="wss://main.devnet.fractalprotocol.com:443" \
  foreman start -f explorer/Procfile.dev -d . web
