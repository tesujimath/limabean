use color_eyre::eyre::{bail, Result, WrapErr};
use std::process::Command;

use super::{
    env::{get_deps, Deps},
    jar::locate_jar,
};

pub(crate) fn check_all() -> Result<()> {
    // TODO a failing health check should not stop the others from reporting
    let java = java_health()?;

    let clojure = clojure_health()?;

    let jar = jar_health()?;

    println!("{}\n{}\n{}", &java, &clojure, &jar);

    Ok(())
}

fn java_health() -> Result<String> {
    let java_version = Command::new("java")
        .arg("--version")
        .output()
        .wrap_err("java: not found")?;
    if !java_version.status.success() {
        bail!("java: not available")
    }
    let java_version = String::from_utf8_lossy(java_version.stdout.as_slice());
    Ok(format!("java: {}", java_version.replace("\n", "; ")))
}

fn clojure_health() -> Result<String> {
    let clojure_version = Command::new("clojure")
        .arg("--version")
        .output()
        .map(|op| String::from_utf8_lossy(op.stdout.as_slice()).replace("\n", "; "));

    match get_deps() {
        Deps::Undefined => match clojure_version {
            Ok(description) => Ok(format!(
                "clojure: available but not required (define $LIMABEAN_DEPS to use): {}",
                description
            )),
            Err(_) => Ok("clojure: not required because $LIMABEAN_DEPS undefined".to_string()),
        },
        Deps::DefinedButUnavailable(path) => {
            bail!("$LIMABEAN_DEPS is {} which cannot be read", &path)
        }
        Deps::Available(path) => match clojure_version {
            Ok(description) => Ok(format!("clojure: {}", description)),
            Err(_) => bail!("$LIMABEAN_DEPS is {} but can't find clojure", &path),
        },
    }
}

fn jar_health() -> Result<String> {
    let jar_path = locate_jar()?;

    Ok(format!("jar: {}", &jar_path))
}
