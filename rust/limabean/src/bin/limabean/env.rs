use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

const DEPS_ENV: &str = "LIMABEAN_DEPS";

#[derive(Copy, Clone, Debug)]
enum DepsSource {
    XdgConfig,
    Env,
}

impl Display for DepsSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DepsSource::*;

        match self {
            XdgConfig => f.write_str("default"),
            Env => f.write_str(DEPS_ENV),
        }
    }
}

fn infer_src_dir(deps: &Path) -> PathBuf {
    deps.parent()
        .unwrap_or_else(|| {
            panic!(
                "Couldn't determine parent directory of {}",
                deps.to_string_lossy()
            )
        })
        .join("src")
}

#[derive(Clone, Debug)]
pub(crate) struct Deps {
    deps_edn: PathBuf,
    source: DepsSource,
    exists: bool,
    src_dir: PathBuf,
    user_clj: PathBuf,
}

impl Deps {
    pub(crate) fn new() -> Deps {
        if let Ok(deps_path) = std::env::var(DEPS_ENV) {
            Deps::from_deps_edn_path(deps_path.into(), DepsSource::Env)
        } else {
            let config_dir = xdg::BaseDirectories::with_prefix("limabean")
                .get_config_home()
                .unwrap_or_else(|| panic!("Couldn't determine XDG_CONFIG_HOME, is HOME defined?"));
            Deps::from_deps_edn_path(
                config_dir.join("clj").join("deps.edn"),
                DepsSource::XdgConfig,
            )
        }
    }

    fn from_deps_edn_path(path: PathBuf, source: DepsSource) -> Self {
        let exists = File::open(&path).is_ok();
        let src_dir = infer_src_dir(&path);
        let user_clj = src_dir.join("user.clj");
        Deps {
            deps_edn: path,
            source,
            exists,
            src_dir,
            user_clj,
        }
    }

    pub(crate) fn exists(&self) -> bool {
        self.exists
    }

    pub(crate) fn path(&self) -> &Path {
        &self.deps_edn
    }

    pub(crate) fn src_dir(&self) -> &Path {
        &self.src_dir
    }

    pub(crate) fn user_clj_exists(&self) -> bool {
        File::open(&self.user_clj).is_ok()
    }

    pub(crate) fn user_clj(&self) -> &Path {
        &self.user_clj
    }

    pub(crate) fn explain_missing(&self) -> String {
        use DepsSource::*;

        match self.source {
            XdgConfig => format!(
                "limabean can't read default deps.edn file at
{}

To bootstrap the Clojure environment for limabean, run `limabean bootstrap`,
which will create deps.edn along with an initial Clojure file in
{}
which is where you can add your own functions.

Alternatively, if you would like your deps.edn to be somewhere else, define the
environment variable {} before running `limabean bootstrap`.",
                self.deps_edn.to_string_lossy(),
                self.user_clj.to_string_lossy(),
                DEPS_ENV
            ),
            Env => format!(
                "Environment variable {} is defined as
{}
but this file does not exist.

To bootstrap the Clojure environment for limabean, run `limabean bootstrap`,
which will create deps.edn along with an initial Clojure file in
{}
which is where you can add your own functions.",
                DEPS_ENV,
                self.deps_edn.to_string_lossy(),
                self.user_clj.to_string_lossy()
            ),
        }
    }

    pub(crate) fn create_dirs(&self) {
        std::fs::create_dir_all(self.deps_edn.parent().unwrap()).unwrap();
        std::fs::create_dir_all(self.user_clj.parent().unwrap()).unwrap();
    }
}
