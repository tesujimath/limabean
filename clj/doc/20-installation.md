# Installation

Firstly, the [Clojure CLI](https://clojure.org/reference/clojure_cli) is required to be installed separately, and `clojure` must be on the user's path.

Once the two binaries `limabean` and `limabean-pod` are on the path, the corresponding `limabean` Clojure code is downloaded automatically on first run from [Clojars](https://clojars.org/io.github.tesujimath/limabean/).

Options for installing these binaries:

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
