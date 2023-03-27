#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

rm -rf dist
mkdir dist
mkdir -p public

pushd ../valuescript_wasm
  wasm-pack build
popd

cp ../valuescript_wasm/pkg/valuescript_wasm_bg.wasm public/value_script_bg.wasm

npm install
npm run build
