use clap::{Parser, Subcommand};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};
use tabulator::Cell;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use limabean::api;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the JSON-RPC server
    Serve {
        /// Beancount file path
        beanfile: PathBuf,
    },

    /// Tabulate JSON according to tabulator
    Tabulate,
}

const LIMABEAN_POD_LOG: &str = "LIMABEAN_POD_LOG";
const LIMABEAN_POD_LOG_LEVEL: &str = "LIMABEAN_POD_LOG_LEVEL";

fn main() {
    // enable logging only if environment variable LIMABEAN_POD_LOG defined
    // log level set via environment variable LIMABEAN_POD_LOG_LEVEL, or default
    if let Ok(log_path) = std::env::var(LIMABEAN_POD_LOG) {
        let log_path: PathBuf = log_path.into();

        if let Some(log_file_name) = log_path.file_name() {
            let log_dir = log_path.parent().unwrap_or(Path::new("."));
            let appender = tracing_appender::rolling::never(log_dir, log_file_name);
            let env_filter = EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .with_env_var(LIMABEAN_POD_LOG_LEVEL)
                .from_env_lossy();
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(appender);

            tracing::subscriber::set_global_default(subscriber.finish()).unwrap();
        }
    }

    let cli = Cli::parse();

    if let Err(e) = match &cli.command {
        Command::Serve { beanfile } => {
            api::serve(beanfile);
            Ok(())
        }

        Command::Tabulate => tabulate(),
    } {
        eprintln!("limabean-pod {}", &e);
        std::process::exit(1);
    }
}

fn tabulate() -> Result<(), crate::Error> {
    let mut input = String::new();

    io::stdin().read_to_string(&mut input)?;

    match Cell::from_json(&input) {
        Ok(cell) => {
            println!("{}", &cell);
            Ok(())
        }
        Err(e) => Err(crate::Error::JsonDecode(e, input)),
    }
}

pub(crate) mod errors;
pub(crate) use errors::Error;
