use clap::{Parser, Subcommand, ValueEnum};
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

    /// Calculate all the bookings
    Book {
        /// Beancount file path
        beanfile: PathBuf,

        /// Output format, defaults to beancount
        #[clap(short)]
        format: Option<Format>,
    },

    /// Tabulate JSON according to tabulator
    Tabulate,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum Format {
    #[default]
    Beancount,
    Edn,
}

impl From<Format> for book::Format {
    fn from(value: Format) -> Self {
        use Format::*;
        use book::Format as B;

        match value {
            Beancount => B::Beancount,
            Edn => B::Edn,
        }
    }
}

const LIMABEAN_POD_LOG: &str = "LIMABEAN_POD_LOG";
const LIMABEAN_POD_LOG_LEVEL: &str = "LIMABEAN_POD_LOG_LEVEL";

fn main() {
    let out_w = &std::io::stdout();
    let error_w = &std::io::stderr();

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

        Command::Book { beanfile, format } => book::write_bookings_from(
            beanfile,
            format.unwrap_or(Format::default()).into(),
            out_w,
            error_w,
        ),

        Command::Tabulate => tabulate(),
    } {
        use crate::Error::*;

        match e {
            FatalAndAlreadyExplained => (),
            _ => eprintln!("limabean-pod {}", &e),
        }

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

pub(crate) mod book;
pub(crate) mod errors;
pub(crate) use errors::Error;
pub(crate) mod format;
pub(crate) mod options;
pub(crate) mod plugins;
