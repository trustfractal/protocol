#!/usr/bin/env bash

set -o errexit
set -o pipefail

if [ ! -d $OUTPUT_PATH ]; then
  mkdir -p $OUTPUT_PATH
fi

output_path=$OUTPUT_PATH/pipeline-parameters.json
head=$CIRCLE_SHA1
base=$(git merge-base master $head)

# If we're building on master, try to use HEAD~1
# This might fail for the first commit, so instead use
# the magic SHA-1 hash for the empty tree
if [[ "$head" == "$base" ]]; then
  if ! base=$(git rev-parse HEAD~1 2>/dev/null); then
    base="4b825dc642cb6eb9a060e54bf8d69288fbee4904"
  fi
fi

diff=$(git diff --name-only "$base" "$head")

for d in $diff; do
  [[ $d == data_host/chrome_extension/* ]] && \
    run_chrome_ext_build=${run_chrome_ext_build:-true}

  [[ $d == blockchain/* ]] && \
    run_rust_build=${run_rust_build:-true}

  [[ $d == support/* ]] && \
    run_rust_build=${run_rust_build:-true}

  [[ $d == .circleci/* ]] && \
    run_rust_build=${run_rust_build:-true} && \
    run_chrome_ext_build=${run_chrome_ext_build:-true}
done

run_rust_build=${run_rust_build:-false}
run_chrome_ext_build=${run_chrome_ext_build:-false}

build_parameters="{\"run-chrome-ext-build\": $run_chrome_ext_build, \"run-rust-build\": $run_rust_build}"

echo "$build_parameters" > $output_path
