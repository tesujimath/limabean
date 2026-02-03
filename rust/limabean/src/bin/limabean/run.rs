use std::{ffi::OsStr, process::Command};

use super::env::Deps;

fn run_or_fail_with_message(mut cmd: Command) {
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

pub(crate) fn run(args: &[String]) {
    let deps = Deps::new();
    if !deps.exists() {
        eprintln!("{}", deps.explain_missing());
        std::process::exit(1);
    }

    let mut clojure_cmd = Command::new("clojure"); // use clojure not clj to avoid rlwrap
    clojure_cmd
        .arg("-Sdeps")
        .arg(deps.path().to_string_lossy().as_ref())
        .arg("-M")
        .arg("-m")
        .arg("limabean.main")
        .args(
            args.iter()
                .map(|s| OsStr::new(s.as_str()))
                .collect::<Vec<_>>(),
        );

    run_or_fail_with_message(clojure_cmd)
}
