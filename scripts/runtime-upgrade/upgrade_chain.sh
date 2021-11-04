#! /bin/bash
SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
ROOT_DIR="$SCRIPT_DIR/../../"
(cd $ROOT_DIR ; cargo build --release)
(cd $SCRIPT_DIR ; yarn)
node $SCRIPT_DIR/index.js --nodeAddress $1 --privateKey $2 --wasmPath $ROOT_DIR/target/release/wbuild/fractal-protocol-blockchain-runtime/fractal_protocol_blockchain_runtime.wasm
