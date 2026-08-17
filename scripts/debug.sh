#!/usr/bin/env sh

set -e

cargo build --package=catsquad-web --lib --target=wasm32-unknown-unknown --profile wasm_debug
cargo build --package=catsquad-api --features test_backdoors

rm -rf ./target/dist/*
mkdir -p ./target/dist
cp -r ./assets/* ./target/dist

tailwindcss -i style/tailwind.css -o target/dist/catsquad.css
wasm-bindgen ./target/wasm32-unknown-unknown/wasm_debug/catsquad_web.wasm --no-typescript --target no-modules --out-dir ./target/dist --out-name catsquad

RUST_LOG="cat=trace" LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:./target/debug/deps/" ./target/debug/catsquad-api

