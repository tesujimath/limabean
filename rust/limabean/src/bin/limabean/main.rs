use tracing_subscriber::EnvFilter;

fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args = std::env::args().collect::<Vec<_>>();

    if let Some("health") = args.get(1).map(String::as_str) {
        check_all(true);
    } else if let Some("bootstrap") = args.get(1).map(String::as_str) {
        check_clojure(false);
        bootstrap::create_files();
        run::run(&["--help".to_string()]);
    } else {
        check_deps(false);
        run::run(&args[1..]);
    }
}

mod bootstrap;
mod env;
mod health;
use health::{check_all, check_clojure, check_deps};
mod run;
