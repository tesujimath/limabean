use clap::{Parser, Subcommand, ValueEnum};
use std::{
    io::{self, Read},
    path::PathBuf,
};
use tabulator::Cell;
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

fn main() {
    let out_w = &std::io::stdout();
    let error_w = &std::io::stderr();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = Cli::parse();

    if let Err(e) = match &cli.command {
        Command::Serve { beanfile } => {
            api::serve(beanfile).map_err(|e| Error::Unexpected(Box::new(e)))
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
