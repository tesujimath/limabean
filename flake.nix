{
  description = "A development environment flake for limabean.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    autobean-format = {
      url = "github:SEIAROTg/autobean-format";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs {
            inherit system;
          };
          pkgs-with-rust-overlay = import inputs.nixpkgs {
            inherit system overlays;
          };
          flakePkgs = {
            autobean-format = inputs.autobean-format.packages.${system}.default;
          };
          # cargo-nightly based on https://github.com/oxalica/rust-overlay/issues/82
          nightly = pkgs-with-rust-overlay.rust-bin.selectLatestNightlyWith (t: t.default);
          cargo-nightly = pkgs.writeShellScriptBin "cargo-nightly" ''
            export RUSTC="${nightly}/bin/rustc";
            exec "${nightly}/bin/cargo" "$@"
          '';

          ci-packages = with pkgs; [
            bashInteractive
            coreutils
            diffutils
            just

            cargo
            gcc

            clojure
            neil
            git
          ];

          limabean =
            let cargo = builtins.fromTOML (builtins.readFile ./rust/limabean/Cargo.toml);
            in pkgs.rustPlatform.buildRustPackage
              {
                pname = "limabean";
                version = cargo.package.version;

                src = ./rust;

                cargoDeps = pkgs.rustPlatform.importCargoLock {
                  lockFile = ./rust/Cargo.lock;
                };

                meta = with pkgs.lib; {
                  description = "Beancount frontend using Rust and Clojure and the Lima parser";
                  homepage = "https://github.com/tesujimath/limabean";
                  license = with licenses; [ asl20 mit ];
                  # maintainers = [ maintainers.tesujimath ];
                };
              };

        in
        with pkgs;
        {
          devShells.default = mkShell {
            nativeBuildInputs = [
              cargo-modules
              cargo-nightly
              cargo-udeps
              cargo-outdated
              cargo-edit
              gdb

              # useful tools:
              beancount
              beanquery
              flakePkgs.autobean-format
            ] ++ ci-packages;

            shellHook = ''
              PATH=$PATH:$(pwd)/scripts.dev:$(pwd)/rust/target/debug

              export LIMABEAN_CLJ_LOCAL_ROOT=$(pwd)/clj
              export LIMABEAN_USER_CLJ=$(pwd)/examples/clj/user.clj
              export LIMABEAN_BEANFILE=$(pwd)/examples/beancount/full.beancount
              export LIMABEAN_LOG=$(pwd)/limabean.log
            '';
          };

          packages.default = limabean;

          apps = {
            tests = {
              type = "app";
              program = "${writeShellScript "limabean-tests" ''
                export PATH=${pkgs.lib.makeBinPath ci-packages}
                just test
              ''}";
            };
          };
        }
      );
}
