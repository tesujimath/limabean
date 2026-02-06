use std::{ffi::OsStr, process::Command};

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

const LIMABEAN_CLJ_LOCAL_ROOT: &str = "LIMABEAN_CLJ_LOCAL_ROOT";

/// deps arg, either from LIMABEAN_CLJ_LOCAL_ROOT or default from Clojars for the matching version
fn deps() -> String {
    let limabean_coord = if let Ok(local_root) = std::env::var(LIMABEAN_CLJ_LOCAL_ROOT) {
        format!(r###"{{:local/root "{}"}}"###, &local_root,)
    } else {
        let version = env!("CARGO_PKG_VERSION");

        format!(r###"{{:mvn/version "{}"}}"###, version,)
    };

    format!(
        r###"{{:deps {{io.github.tesujimath/limabean {}}}}}
"###,
        limabean_coord
    )
}

pub(crate) fn run(args: &[String]) {
    let verbose = args.iter().any(|arg| arg == "-v" || arg == "--verbose");

    let mut clojure_cmd = Command::new("clojure"); // use clojure not clj to avoid rlwrap
    clojure_cmd
        .arg("-Sdeps")
        .arg(deps())
        .arg("-M")
        .arg("-m")
        .arg("limabean.main")
        .args(
            args.iter()
                .map(|s| OsStr::new(s.as_str()))
                .collect::<Vec<_>>(),
        );

    if verbose {
        eprintln!("{:?}", &clojure_cmd);
    }

    run_or_fail(clojure_cmd)
}
