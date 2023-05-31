#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR"

pushd ../valuescript_wasm
  wasm-pack build
popd

cp ../valuescript_wasm/pkg/valuescript_wasm_bg.wasm public/value_script_bg.wasm
