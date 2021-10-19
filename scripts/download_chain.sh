#! /usr/bin/bash

set -euxo pipefail

sub2sql --chain=wss://nodes.mainnet.fractalprotocol.com \
  --out=mainnet.sqlite \
  --types="$(cat blockchain/types.json | tr -d '\n')"
