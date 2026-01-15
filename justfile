build: rust-build

test: rust-test clj-test

[working-directory: 'rust']
rust-build:
    cargo build

[working-directory: 'rust']
rust-test: rust-build
    cargo test

[working-directory: 'clj']
clj-test:
    neil test
