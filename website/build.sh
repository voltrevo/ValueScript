#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

rm -rf dist
mkdir dist
mkdir dist/monaco
mkdir dist/monaco/workers
mkdir dist/monaco/pieces
mkdir dist/playground

deno run --allow-all --quiet 'https://deno.land/x/packup@v0.2.2/cli.ts' \
  build src/playground/index.html

mv dist/*.* dist/playground/.

deno run --allow-all --quiet 'https://deno.land/x/packup@v0.2.2/cli.ts' \
  build src/index.html

pushd ../valuescript_wasm
  wasm-pack build
popd

cp ../valuescript_wasm/pkg/valuescript_wasm_bg.wasm dist/value_script_bg.wasm

tar xf monaco.tar.gz -C dist/.
