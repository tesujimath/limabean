use color_eyre::eyre::{self, Result};
use real_parent::PathExt;
use std::{borrow::Cow, fs::File};

use super::env;

pub(crate) fn locate_jar() -> Result<String> {
    let version = env!("CARGO_PKG_VERSION");
    let jar_candidate_relpaths = [
        // deployed:
        format!("../lib/limabean-{}.jar", version),
        // development with real version:
        format!(
            "../../../clj/target/net.clojars.limabean/limabean-{}.jar",
            version
        ),
        // development with snapshot version:
        format!(
            "../../../clj/target/net.clojars.limabean/limabean-{}-SNAPSHOT.jar",
            version
        ),
    ];

    let exe_dir = env::exe_dir()?;
    let jar_candidates = jar_candidate_relpaths
        .into_iter()
        .filter_map(|relpath| exe_dir.join(relpath).real_clean().ok())
        .collect::<Vec<_>>();

    let jar = jar_candidates
        .iter()
        .find(|path| File::open(path).is_ok())
        .ok_or(eyre::format_err!(
            "can't find jarfile, tried {}",
            itertools::Itertools::intersperse(
                jar_candidates.iter().map(|path| path.to_string_lossy()),
                Cow::Borrowed(", ")
            )
            .collect::<String>()
        ))?;

    Ok(jar.to_string_lossy().to_string())
}
