use color_eyre::eyre::{bail, Result, WrapErr};
use std::{ffi::OsStr, fmt::Display, process::Command};

use super::{
    env::{get_deps, Deps},
    jar::locate_jar,
};

fn run_or_fail_with_message<S>(mut cmd: Command, error_message: S) -> Result<()>
where
    S: Display + Sync + Send + 'static,
{
    let exit_status = cmd
        .spawn()
        .wrap_err(error_message)?
        .wait()
        .unwrap_or_else(|e| panic!("Failed to wait: {}", e));

    // any error message is already written on stderr, so we're done
    // TODO improve error path here, early exit is nasty
    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }
    Ok(())
}

pub(crate) fn run(args: &[String]) -> Result<()> {
    match get_deps() {
        Deps::Undefined => {
            // run with Java
            let jar = locate_jar()?;
            let mut java_cmd = Command::new("java");
            java_cmd.arg("-jar").arg(&jar).args(
                args.iter()
                    .map(|s| OsStr::new(s.as_str()))
                    .collect::<Vec<_>>(),
            );
            run_or_fail_with_message(java_cmd, "java: not found")
        }
        Deps::DefinedButUnavailable(path) => {
            bail!("Fatal error: cannot read $LIMABEAN_DEPS={}", &path);
        }
        Deps::Available(deps_path) => {
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
    }
}
