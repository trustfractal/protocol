#! /bin/bash

set -euxo pipefail

DIR=$(mktemp -d)
git clone https://github.com/shelbyd/sanity $DIR

cargo run --manifest-path $DIR/Cargo.toml -- deploy

rm -rf $DIR
