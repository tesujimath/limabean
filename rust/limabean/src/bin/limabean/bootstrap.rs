use std::{
    borrow::Cow,
    fmt::Display,
    io::{self, Write},
};

use super::env::Deps;

fn confirm<S>(message: S) -> bool
where
    S: Display,
{
    print!("{} [y/n] ", message);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub(crate) fn create_files() {
    let deps = Deps::new();

    let mut paths_to_create: Vec<Cow<'_, str>> = Vec::default();

    let deps_path = deps.path();
    let create_deps = if !deps.exists() {
        paths_to_create.push(deps_path.to_string_lossy());
        true
    } else {
        eprintln!(
            "Not creating {}, file already exists",
            deps_path.to_string_lossy()
        );
        false
    };

    let create_user_clj = if !deps.user_clj_exists() {
        paths_to_create.push(deps.user_clj().to_string_lossy());
        true
    } else {
        eprintln!(
            "Not creating {}, file already exists",
            deps.user_clj().to_string_lossy()
        );
        false
    };

    if create_deps || create_user_clj {
        if confirm(format!("Will create {}, ok?", paths_to_create.join(", "))) {
            deps.create_dirs();

            if create_deps {
                // version number of limabean on Clojars matches version of limabean crate
                let version = env!("CARGO_PKG_VERSION");
                let deps = format!(
                    r###"{{:deps {{io.github.tesujimath/limabean {{:mvn/version "{}"}}}},
 :paths ["src"]}}
"###,
                    version
                );
                std::fs::write(deps_path, deps).expect("Failed to write deps.edn");
            }

            if create_user_clj {
                let user_clj = r###"(ns user
  (:require [java-time.api :as jt]
            [limabean.core.filters :as f]))

(defn fy
  "Example of financial year date filter"
  [year]
  (let [year (if (< year 100) (+ 2000 year) year)]
    (f/every-f (f/date>= (jt/local-date year 4 1))
               (f/date< (jt/local-date (inc year) 4 1)))))
"###;

                std::fs::write(deps.user_clj(), user_clj).expect("Failed to write user.clj");
            }
        } else {
            println!("abort");
        }
    } else {
        println!("Nothing to do")
    }
}
