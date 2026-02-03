use std::process::Command;

use super::env::Deps;

#[derive(Clone, Debug)]
enum Health {
    Good(String),
    Bad(String),
}

pub(crate) fn check_all(verbose: bool) {
    check_clojure(verbose);
    check_deps(verbose);
}

pub(crate) fn check_clojure(verbose: bool) {
    match clojure_health() {
        Health::Good(description) => {
            if verbose {
                println!("{}", description);
            }
        }
        Health::Bad(reason) => {
            eprintln!("limabean {reason}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn check_deps(verbose: bool) {
    let deps = Deps::new();
    if deps.exists() {
        if verbose {
            println!("deps.edn at {}", deps.path().to_string_lossy());
        }
    } else {
        eprintln!("{}", deps.explain_missing());
        std::process::exit(1);
    }
}

fn clojure_health() -> Health {
    match Command::new("clojure")
        .arg("--version")
        .output()
        .map(|op| String::from_utf8_lossy(op.stdout.as_slice()).replace("\n", "; "))
    {
        Ok(description) => Health::Good(format!("clojure: {}", description)),
        Err(e) => Health::Bad(format!("can't find clojure: {}", &e)),
    }
}
