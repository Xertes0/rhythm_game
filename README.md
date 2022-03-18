# Rhythm Game
! This is just a prototype !

## Run native
`cargo run`

## Compile to wasm
`cargo build --target wasm32-unknown-unknown`

## Compile to android
`docker run --rm -v $(pwd)":/root/src" -w /root/src notfl3/cargo-apk cargo quad-apk build`
