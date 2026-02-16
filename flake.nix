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

          version = (builtins.fromTOML (builtins.readFile ./rust/limabean/Cargo.toml)).package.version;
          limabean = let inherit (pkgs) clojure lib makeWrapper rustPlatform; in
            rustPlatform.buildRustPackage
              {
                inherit version;

                pname = "limabean";

                src = ./rust;

                cargoDeps = rustPlatform.importCargoLock {
                  lockFile = ./rust/Cargo.lock;
                };

                meta = with lib; {
                  description = "Beancount frontend using Rust and Clojure and the Lima parser";
                  homepage = "https://github.com/tesujimath/limabean";
                  license = with licenses; [ asl20 mit ];
                  # maintainers = [ maintainers.tesujimath ];
                };

                nativeBuildInputs = [ makeWrapper ];

                propagatedBuildInputs = [
                  clojure
                ];
              };

          limabean-clj =
            let inherit (pkgs) cacert cargo clojure git lib stdenv writeShellScriptBin;
              mavenRepoSha256 = "sha256-mO98ssHyCAI1LPR5qPlRtH6ygd3h2xRP381K/zLdVSY=";
              src = ./clj;

              setupClojureEnv = ''
                export HOME="$(mktemp -d)"
                export GITLIBS="$HOME/.gitlibs"
                export GIT_SSL_CAINFO=${cacert}/etc/ssl/certs/ca-bundle.crt
              '';

              mavenRepo = stdenv.mkDerivation {
                name = "limabean-${version}-maven-deps";
                inherit src;

                nativeBuildInputs = [ clojure git ];

                buildPhase = ''
                  ${setupClojureEnv}

                  runHook preBuild

                  mkdir -p "$out"

                  # -P       -> resolve all normal deps
                  # -M:alias -> resolve extra-deps of the listed aliases
                  clj -Sdeps "{:mvn/local-repo \"$out\"}" -P -M:build

                  runHook postBuild
                '';

                # copied from buildMavenPackage
                # keep only *.{pom,jar,sha1,nbm} and delete all ephemeral files with lastModified timestamps inside
                installPhase = ''
                  runHook preInstall

                  find $out -type f \( \
                    -name \*.lastUpdated \
                    -o -name resolver-status.properties \
                    -o -name _remote.repositories \) \
                    -delete

                  runHook postInstall
                '';

                dontFixup = true;

                outputHash = mavenRepoSha256;
                outputHashMode = "recursive";
                outputHashAlgo = "sha256";
              };

              clojureWithCache = writeShellScriptBin "clojure" ''
                exec ${lib.getExe' clojure "clojure"} -Sdeps '{:mvn/local-repo "${mavenRepo}"}' "$@"
              '';
            in
            stdenv.mkDerivation
              {
                inherit src version;

                pname = "limabean-clj";

                nativeBuildInputs = [ cargo clojureWithCache git ];

                buildPhase = ''
                  ${setupClojureEnv}

                  runHook preBuild

                  type git
                  echo "HOME is $HOME"
                  ls -lad $HOME
                  ls -la $HOME
                  mkdir -p "$HOME/.m2/repository"
                  ls -la $HOME/.m2
                  echo "GITLIBS is $GITLIBS"
                  export GIT="${git}/bin/git"
                  echo "GIT is $GIT"
                  echo "mavenRepo is ${mavenRepo}"

                  # this needs cargo to get the version from Cargo.toml:
                  # echo clojure -Sdeps "{:mvn/local-repo \"${mavenRepo}\"}" -T:build uber
                  # clojure -Sdeps "{:mvn/local-repo \"${mavenRepo}\"}" -T:build uber
                  clojure -T:build uber

                  runHook postBuild
                '';

                installPhase = ''
                  runHook preInstall

                  mkdir -p $out/share/limabean

                  echo "build dir"
                  pwd
                  echo "content of build dir"
                  ls -l
                  echo "content of build/target dir"
                  ls -l target
                  
                  install -Dm644 "target/limabean-${version}-standalone.jar" $out/share/limabean

                  runHook postInstall
                '';

                meta = with lib; {
                  description = "limabean uberjar";
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
              clippy
              rustc

              jre
              # useful tools:
              beancount
              beanquery
              flakePkgs.autobean-format
            ] ++ ci-packages;

            shellHook = ''
              PATH=$PATH:$(pwd)/scripts.dev:$(pwd)/rust/target/debug

              export LIMABEAN_UBERJAR=$(pwd)/clj/target/limabean-${version}-standalone.jar
              export LIMABEAN_CLJ_LOCAL_ROOT=$(pwd)/clj
              export LIMABEAN_USER_CLJ=$(pwd)/examples/clj/user.clj
              export LIMABEAN_BEANFILE=$(pwd)/test-cases/full.beancount
              export LIMABEAN_LOG=$(pwd)/limabean.log
            '';
          };

          packages = { inherit limabean limabean-clj; default = limabean; };

          apps = {
            tests = {
              type = "app";
              program = "${writeShellScript "limabean-tests" ''
                export PATH=${pkgs.lib.makeBinPath ci-packages}:$(pwd)/rust/target/debug
                just test
              ''}";
            };
          };
        }
      );
}
