use color_eyre::eyre::{Result, WrapErr};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub(crate) enum Deps {
    Undefined,
    DefinedButUnavailable(String),
    Available(String),
}

pub(crate) fn get_deps() -> Deps {
    if let Ok(path) = std::env::var("LIMABEAN_DEPS") {
        if File::open(&path).is_ok() {
            Deps::Available(path)
        } else {
            Deps::DefinedButUnavailable(path)
        }
    } else {
        Deps::Undefined
    }
}

/// Return dir where this executable resides
pub(crate) fn exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().wrap_err("can't determine executable path")?;
    let exe_absdir = exe_path.parent().expect("no parent for exe");

    Ok(exe_absdir.to_path_buf())
}
