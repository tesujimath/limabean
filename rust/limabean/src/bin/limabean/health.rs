use std::process::Command;

use super::env::Deps;

#[derive(Clone, Debug)]
enum Health {
    Good(String),
    Bad(String),
}

pub(crate) fn check_all() {
    let mut failed = false;

    match clojure_health() {
        Health::Good(description) => {
            println!("{}", description);
        }
        Health::Bad(reason) => {
            eprintln!("limabean {reason}");
            failed = true;
        }
    }

    let deps = Deps::new();
    if deps.exists() {
        println!("deps.edn at {}", deps.path().to_string_lossy());
    } else {
        eprintln!("{}", deps.explain_missing());
        failed = true;
    }

    if failed {
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
