# Running the development version

`limabean` supports running from a local copy of the repo.  Simply set the environment variable `LIMABEAN_CLJ_LOCAL_ROOT` to the path of the `clj` directory.  Passing the `-v` or `--verbose` flag reveals what is happening.

```
kiri> echo $LIMABEAN_CLJ_LOCAL_ROOT

kiri> limabean -v --beanfile ./examples/beancount/simple.beancount
"clojure" "-Sdeps" "{:deps {io.github.tesujimath/limabean {:mvn/version \"0.1.0\"}}}\n" "-M" "-m" "limabean.main" "-v" "--beanfile" "./examples/beancount/simple.beancount"


kiri> echo $LIMABEAN_CLJ_LOCAL_ROOT
/Users/sjg/vc/tesujimath/limabean/clj

kiri> ls $LIMABEAN_CLJ_LOCAL_ROOT
CHANGELOG.md  README.md  build.clj  deps.edn  doc/  resources/  src/  target/  test/

kiri> limabean -v --beanfile ./examples/beancount/simple.beancount
"clojure" "-Sdeps" "{:deps {io.github.tesujimath/limabean {:local/root \"/Users/sjg/vc/tesujimath/limabean/clj\"}}}\n" "-M" "-m" "limabean.main" "-v" "--beanfile" "./examples/beancount/simple.beancount"
```

Also, since the `limabean` does nothing beyond launching the Clojure code, it is also possible to dispense with it altogether and run purely from the project directory, for example:

```
kiri> cd $LIMABEAN_CLJ_LOCAL_ROOT
kiri> clojure -M -m limabean.main --beanfile ../examples/beancount/simple.beancount
[Rebel readline] Type :repl/help for online help info
[limabean] 18 directives loaded from ../examples/beancount/simple.beancount
user=>
```

The next level up would be to use a full [Clojure development environment](https://clojure.org/guides/editors), which is highly recommended.
