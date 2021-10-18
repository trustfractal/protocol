#! /bin/bash
cargo build --release
node support/runtime-upgrade/index.js --nodeAddress $1 --rootMnemonicPath $2 --wasmPath ./target/release/wbuild/fractal-protocol-blockchain-runtime/fractal_protocol_blockchain_runtime.wasm
