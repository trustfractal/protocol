#!/bin/bash

NODE_BUILD=$1

# Generate the Spec
docker run --rm  boymaas/nodefcl:$NODE_BUILD build-spec \
  --disable-default-bootnode --chain mainnet >fclMainnetSpec.json

# Generate the Raw Spec
docker run -v $PWD:/data --rm  boymaas/nodefcl:$NODE_BUILD build-spec \
  --chain=/data/fclMainnetSpec.json --raw --disable-default-bootnode >fclMainnetSpecRaw.json

