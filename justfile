build: build-rust build-clj

test: rust-test clj-test

[working-directory: 'rust']
build-rust:
    cargo build

[working-directory: 'rust']
rust-test: build-rust
    cargo test

[working-directory: 'clj']
build-clj:
    clojure -T:build jar

[working-directory: 'clj']
clj-test:
    clojure -X:test
