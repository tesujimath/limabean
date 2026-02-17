# Installation

There are two ways to run `limabean`, either standalone or from Clojars.

Running from Clojars is recommended for anyone using the [GitHub release](https://github.com/tesujimath/limabean/releases), that is, not setting any of the following environment variables.

Selection of runtime is determined by the following:

1. If the environment variable `LIMABEAN_CLJ_LOCAL_ROOT` is defined at runtime, that is the path to local Clojure source, and is used to run the [development version](50-development.md) using `clojure`
2. If the environment variable `LIMABEAN_UBERJAR` is defined at runtime, that is the path to the standalone application jarfile, which is run using `java`
3. If the environment variable `LIMABEAN_UBERJAR` was defined at buildtime, that is the path to the standalone application jarfile, which is run using `java`
4. Otherwise, the application whose version matches `limabean` is run from Clojars using `clojure`

Note that running using `clojure` will download all required dependencies from Clojars.

## Running from Clojars

Requirements:

1. The [Clojure CLI](https://clojure.org/reference/clojure_cli) is required to be installed separately, and `clojure` must be on the user's path.

2. The two Rust binaries `limabean` and `limabean-pod` must be installed and on the path.

The corresponding `limabean` Clojure code is downloaded automatically on first run from [Clojars](https://clojars.org/io.github.tesujimath/limabean/).

Options for installing the Rust binaries:

1. Tarballs and zipfiles are provided for each [GitHub release](https://github.com/tesujimath/limabean/releases) for Linux, macOS, and Windows

2. If you have a Rust toolchain installed, `cargo install limabean` will install the two binaries `limabean` and `limabean-pod` into `~/.cargo/bin`.  Add this directory to your path before running `limabean`

3. If you have Nix, `limabean` is available as a Nix flake at `url = "github:tesujimath/limabean"`, and this flake pulls in the Clojure CLI tools automatically

### macOS

On macOS it is necessary to remove the quarantine attributes after unpacking the tarball, e.g.

```
xattr -rd com.apple.quarantine ./limabean/bin
```

### Windows

- install [OpenJDK 25 MSI](https://learn.microsoft.com/en-us/java/openjdk/download)
- install [Clojure 1.12 MSI](https://github.com/casselc/clj-msi)
- download limabean zipfile from GitHub releases and extract somewhere
- add that directory to path

## Standalone

Requirements:

1. Java runtime installed separately, with `java` on the user's path.  Note that the `java.sql` module at least is required, so a minimal jre may be insufficient.

2. The two Rust binaries `limabean` and `limabean-pod` must be installed and on the path.

3. The limabean standalone jarfile must be available at a location given by the environment variable `LIMABEAN_UBERJAR`

If this environment variable is defined when building the Rust binaries, it is not required at runtime, which is recommended when packaging `limabean`.

## Building from source

The [`justfile`](../../justfile) has recipes for building from source.

For packagers wishing to build a standalone jarfile, `build-standalone-release` is the rule to use.  The two Rust binaries will be built in `rust/target/release`, and the standalone jarfile will be built in `clj/target`.
