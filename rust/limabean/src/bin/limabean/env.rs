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

// return longest prefix and its length in path components
fn longest_prefix<P1, P2>(a: P1, b: P2) -> (PathBuf, usize)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let components = a
        .as_ref()
        .components()
        .zip(b.as_ref().components())
        .take_while(|(x, y)| x == y)
        .map(|(x, _)| x)
        .collect::<Vec<_>>();

    let n = components.len();
    let prefix = components.into_iter().collect::<PathBuf>();

    (prefix, n)
}

/// Return dir where this executable resides, as a relative path if possible
pub(crate) fn exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().wrap_err("can't determine executable path")?;
    let exe_absdir = exe_path.parent().expect("no parent for exe");

    let (prefix, prefix_len) = longest_prefix(exe_absdir, std::env::current_dir().expect("no cwd"));

    // if there's any reasonable common prefix between cwd and the exe_absdir, prefer to use relative paths
    // 2 is entirely arbitrary, but means if both are in say /home/<user> we'll be using relative paths
    if prefix_len > 2 {
        let exe_reldir = exe_absdir.strip_prefix(prefix).unwrap();

        // empty path doesn't join well, so:
        if exe_reldir.as_os_str().is_empty() {
            Ok(PathBuf::from("."))
        } else {
            Ok(exe_reldir.to_path_buf())
        }
    } else {
        Ok(exe_absdir.to_path_buf())
    }
}
