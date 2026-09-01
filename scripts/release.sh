#!/usr/bin/env sh

set -e

cargo build --package=catsquad-web --lib --target=wasm32-unknown-unknown --profile wasm_release
cargo build --package=catsquad-api --release

rm -rf ./target/dist/*
mkdir -p ./target/dist
cp -r ./assets/* ./target/dist

tailwindcss -i style/tailwind.css -o target/dist/catsquad.css
wasm-bindgen ./target/wasm32-unknown-unknown/wasm_release/catsquad_web.wasm --no-typescript --target no-modules --out-dir ./target/dist --out-name catsquad

RUST_LOG="cat=trace" LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:./target/release/deps/" ./target/release/catsquad-api

