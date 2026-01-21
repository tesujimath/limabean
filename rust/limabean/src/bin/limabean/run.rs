use color_eyre::eyre::{bail, Result, WrapErr};
use std::{ffi::OsStr, process::Command};

use super::{
    env::{get_deps, Deps},
    jar::locate_jar,
};

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

            let exit_status = java_cmd
                .spawn()
                .wrap_err("java: not found")?
                .wait()
                .unwrap_or_else(|e| panic!("Failed to wait: {}", e));

            // any error message is already written on stderr, so we're done
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        Deps::DefinedButUnavailable(path) => {
            bail!("Fatal error: cannot read $LIMABEAN_DEPS={}", &path)
        }
        Deps::Available(_path) => {
            // run with clj
            todo!("running with deps.edn not yet supported")
        }
    }
}
