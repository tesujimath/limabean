use std::{ffi::OsStr, process::Command};

pub(crate) enum Runtime {
    Java(String),
    Clojure(String),
}

const LIMABEAN_CLJ_LOCAL_ROOT: &str = "LIMABEAN_CLJ_LOCAL_ROOT";
const LIMABEAN_UBERJAR: &str = "LIMABEAN_UBERJAR";
const LIMABEAN_UBERJAR_BUILDTIME: Option<&str> = option_env!("LIMABEAN_UBERJAR");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Additional Java options
const JVM_OPTIONS: &[&str] = &[
    "--enable-native-access=ALL-UNNAMED", // inhibit warning triggered by JLine
];

impl Runtime {
    fn clojure(limabean_coord: String) -> Self {
        Runtime::Clojure(format!(
            r###"{{:deps {{io.github.tesujimath/limabean {}}}}}"###,
            limabean_coord
        ))
    }

    fn java(uberjar: String) -> Self {
        Runtime::Java(uberjar)
    }

    pub(crate) fn from_env() -> Self {
        if let Ok(local_root) = std::env::var(LIMABEAN_CLJ_LOCAL_ROOT) {
            Runtime::clojure(format!(r###"{{:local/root "{}"}}"###, &local_root))
        } else if let Ok(uberjar) = std::env::var(LIMABEAN_UBERJAR) {
            Runtime::java(uberjar)
        } else if let Some(uberjar) = LIMABEAN_UBERJAR_BUILDTIME {
            Runtime::java(uberjar.to_string())
        } else {
            Runtime::clojure(format!(r###"{{:mvn/version "{}"}}"###, VERSION))
        }
    }

    fn command<S>(&self, args: &[S]) -> Command
    where
        S: AsRef<str>,
    {
        use Runtime::*;

        let mut cmd = match self {
            Java(uberjar) => {
                let mut java_cmd = Command::new("java");
                java_cmd.args(JVM_OPTIONS.iter()).arg("-jar").arg(uberjar);
                java_cmd
            }
            Clojure(deps) => {
                let mut clojure_cmd = Command::new("clojure"); // use clojure not clj to avoid rlwrap
                clojure_cmd
                    .args(JVM_OPTIONS.iter().map(|opt| format!("-J{}", opt)))
                    .arg("-Sdeps")
                    .arg(deps)
                    .arg("-M")
                    .arg("-m")
                    .arg("limabean.main");
                clojure_cmd
            }
        };

        cmd.args(
            args.iter()
                .map(|s| OsStr::new(s.as_ref()))
                .collect::<Vec<_>>(),
        );

        cmd
    }
}

#[cfg(unix)]
fn run_or_fail(mut cmd: Command) {
    use std::os::unix::process::CommandExt;

    let e = cmd.exec(); // on success does not return

    eprintln!(
        "limabean can't run {}: {}",
        cmd.get_program().to_string_lossy(),
        &e
    );
    std::process::exit(1);
}

#[cfg(windows)]
fn run_or_fail(mut cmd: Command) {
    let cmd = cmd
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    match cmd.spawn() {
        Ok(mut child) => {
            let exit_status = child
                .wait()
                .unwrap_or_else(|e| panic!("limabean unexpected wait failure: {}", e));

            // any error message is already written on stderr, so we're done
            // TODO improve error path here, early exit is nasty
            if !exit_status.success() {
                std::process::exit(exit_status.code().unwrap_or(1));
            }
        }

        Err(e) => {
            eprintln!(
                "limabean can't run {}: {}",
                cmd.get_program().to_string_lossy(),
                &e
            );
            std::process::exit(1);
        }
    }
}

pub(crate) fn run(runtime: &Runtime, args: &[String]) {
    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");
    let version = args.iter().any(|arg| arg == "--version");

    if version {
        println!("limabean.rs  {VERSION}");
    }

    let cmd = runtime.command(args);

    if verbose {
        eprintln!("{:?}", &cmd);
    }

    run_or_fail(cmd)
}
