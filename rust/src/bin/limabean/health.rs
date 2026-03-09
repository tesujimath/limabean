use std::{path::Path, process::Command};

use super::run::Runtime;

#[derive(Clone, Debug)]
enum Health {
    Good(String),
    Bad(String),
}

pub(crate) fn check(runtime: &Runtime, verbose: bool) {
    use Runtime::*;

    match runtime {
        Java(uberjar) => {
            check_java(uberjar, verbose);
        }
        Clojure(_) => {
            check_clojure(verbose);
        }
    }
}

fn check_clojure(verbose: bool) {
    match clojure_health() {
        Health::Good(description) => {
            if verbose {
                eprintln!("{}", description);
            }
        }
        Health::Bad(reason) => {
            eprintln!("limabean {reason}");
            std::process::exit(1);
        }
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

fn check_java(uberjar: &str, verbose: bool) {
    match java_health() {
        Health::Good(description) => {
            if verbose {
                eprintln!("{}", description);
            }

            match Path::new(uberjar).try_exists() {
                Ok(false) => {
                    eprintln!("uberjar {} not found", uberjar);
                    std::process::exit(1);
                }
                Ok(true) => {
                    if verbose {
                        eprintln!("uberjar {}", uberjar);
                    }
                }
                Err(e) => {
                    eprintln!("uberjar {} not found: {}", uberjar, &e);
                    std::process::exit(1);
                }
            }
        }
        Health::Bad(reason) => {
            eprintln!("limabean {reason}");
            std::process::exit(1);
        }
    }
}

fn java_health() -> Health {
    match Command::new("java").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                Health::Good(format!(
                    "java: {}",
                    String::from_utf8_lossy(output.stdout.as_slice()).replace("\n", "; ")
                ))
            } else {
                Health::Bad(format!(
                    "java: {}",
                    String::from_utf8_lossy(output.stderr.as_slice()).replace("\n", "; ")
                ))
            }
        }
        Err(e) => Health::Bad(format!("can't find java: {}", &e)),
    }
}
