use color_eyre::eyre::WrapErr;
use std::{ffi::OsStr, fmt::Display, process::Command};

use super::env::Deps;

fn run_or_fail_with_message<S>(mut cmd: Command, error_message: S)
where
    S: Display + Sync + Send + 'static,
{
    let exit_status = cmd
        .spawn()
        .wrap_err(error_message)
        .unwrap()
        .wait()
        .unwrap_or_else(|e| panic!("Failed to wait: {}", e));

    // any error message is already written on stderr, so we're done
    // TODO improve error path here, early exit is nasty
    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
}

pub(crate) fn run(args: &[String]) {
    let deps_path = Deps::new().get_path_or_exit_with_explanation();

    let mut clojure_cmd = Command::new("clojure"); // use clojure not clj to avoid rlwrap
    clojure_cmd
        .arg("-Sdeps")
        .arg(deps_path)
        .arg("-M")
        .arg("-m")
        .arg("limabean.main")
        .args(
            args.iter()
                .map(|s| OsStr::new(s.as_str()))
                .collect::<Vec<_>>(),
        );

    run_or_fail_with_message(clojure_cmd, "clojure: not found")
}
