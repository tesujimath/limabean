build: build-rust build-clj

build-release: build-rust-release build-clj

test: rust-test clj-test

[working-directory: 'rust']
build-rust:
    cargo build

[working-directory: 'rust']
build-rust-release:
    cargo build --release --all-targets

[working-directory: 'rust']
rust-test: build-rust
    cargo test

[working-directory: 'clj']
build-clj:
    clojure -T:build uber

[working-directory: 'clj']
clj-test:
    clojure -X:test
