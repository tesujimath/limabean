use std::{fs::File, path::Path};

pub(crate) enum Deps {
    UnavailableDefault(String, String),
    UnavailableCustom(String, String),
    Available(String),
}

impl Deps {
    pub(crate) fn new() -> Deps {
        if let Ok(deps_path) = std::env::var("LIMABEAN_DEPS") {
            if File::open(&deps_path).is_ok() {
                Deps::Available(deps_path)
            } else {
                let user_clj_path = Path::new(&deps_path)
                    .parent()
                    .unwrap_or_else(|| {
                        panic!(
                            "Couldn't determine parent directory of LIMABEAN_DEPS {}",
                            &deps_path
                        )
                    })
                    .join("src/user.clj");
                Deps::UnavailableCustom(deps_path, user_clj_path.to_string_lossy().into_owned())
            }
        } else {
            let config_dir = xdg::BaseDirectories::with_prefix("limabean")
                .get_config_home()
                .unwrap_or_else(|| panic!("Couldn't determine XDG_CONFIG_HOME, is HOME defined?"));
            let clj_dir = config_dir.join("clj");
            let deps_path = clj_dir.join("deps.edn");
            let deps_path_string = deps_path.to_string_lossy().into_owned();
            if File::open(&deps_path).is_ok() {
                Deps::Available(deps_path_string)
            } else {
                let user_clj_path = clj_dir
                    .join("src")
                    .join("user.clj")
                    .to_string_lossy()
                    .into_owned();

                Deps::UnavailableDefault(deps_path_string, user_clj_path)
            }
        }
    }

    pub(crate) fn get_path_or_exit_with_explanation(self) -> String {
        match self {
            Deps::UnavailableDefault(deps_path, user_clj_path) => {
                eprintln!(
                    "Can't read default deps.edn file at
{}

To bootstrap the limabean Clojure environment, run `limabean bootstrap`,
which will create deps.edn along with an initial Clojure file in
{}
which is where you can add your own functions.

Alternatively, if you would like your deps.edn to be somewhere else, define the
environment variable LIMABEAN_DEPS before running `limabean bootstrap`.",
                    deps_path, user_clj_path
                );
                std::process::exit(1);
            }
            Deps::UnavailableCustom(deps_path, user_clj_path) => {
                eprintln!(
                    "Environment variable LIMABEAN_DEPS is defined as
{}
but this file does not exist.

To bootstrap the limabean Clojure environment, run `limabean bootstrap`,
which will create that deps.edn file along with an initial Clojure file in
{}
which is where you can add your own functions.",
                    deps_path, user_clj_path
                );
                std::process::exit(1);
            }
            Deps::Available(deps_path) => deps_path,
        }
    }
}
