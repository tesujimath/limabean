# Installation

Firstly, the [Clojure CLI](https://clojure.org/reference/clojure_cli) is required to be installed separately, and `clojure` must be on the user's path.

Tarballs and zipfiles are provided for each GitHub release for Linux, macOS, and Windows.  These contain the two Rust binaries `limabean` and `limabean-pod`, which along with the Clojure CLI are all that is needed locally.  The `limabean` Clojure code is downloaded on first run from [Clojars](https://clojars.org/io.github.tesujimath/limabean/).

Alternatively, clone the repo, install a Rust toolchain, and `just build`.

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

Beware [unable to detect a system Terminal on Windows #5](https://github.com/tesujimath/limabean/issues/5)
