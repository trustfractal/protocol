#! /usr/bin/bash

set -euxo pipefail

sub2sql --chain=wss://889e30bc.testnet.fractalprotocol.com \
  --out=testnet.sqlite \
  --types='{"FractalId": "u64", "MerkleTree": "Raw"}'
