use color_eyre::eyre::{Result, WrapErr};
use std::process::Command;

use super::env::Deps;

pub(crate) fn check_all() {
    // if more checks added, a failing health check should not stop the others from reporting
    match clojure_health() {
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
        Ok(clj_status) => {
            println!("{}", clj_status);
        }
    }

    let deps_path = Deps::new().get_path_or_exit_with_explanation();
    println!("Clojure deps.edn: {}", &deps_path);
}

fn clojure_health() -> Result<String> {
    let clojure_version = Command::new("clojure")
        .arg("--version")
        .output()
        .map(|op| String::from_utf8_lossy(op.stdout.as_slice()).replace("\n", "; "))
        .wrap_err("can't find clojure")?;

    Ok(format!("clojure: {}", clojure_version))
}
