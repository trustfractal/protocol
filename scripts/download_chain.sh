#! /usr/bin/bash

set -euxo pipefail

sub2sql --chain=wss://nodes.testnet.fractalprotocol.com \
  --out=testnet.sqlite \
  --types='{"FractalId": "u64", "MerkleTree": "Raw"}'
