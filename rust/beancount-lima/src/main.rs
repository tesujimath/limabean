use color_eyre::eyre::Result;
use std::{
    io::{self, Read},
    path::PathBuf,
    process::exit,
};
use tabulator::Cell;
use tracing_subscriber::EnvFilter;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Calculate all the bookings
    Book {
        /// Beancount file path
        beanpath: PathBuf,

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
        use book::Format as B;
        use Format::*;

        match value {
            Beancount => B::Beancount,
            Edn => B::Edn,
        }
    }
}

fn main() -> Result<()> {
    let out_w = &std::io::stdout();
    let error_w = &std::io::stderr();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = Cli::parse();

    match &cli.command {
        Command::Book { beanpath, format } => book::write_bookings_from(
            beanpath,
            format.unwrap_or(Format::default()).into(),
            out_w,
            error_w,
        ),

        Command::Tabulate => {
            let mut input = String::new();

            if let Err(e) = io::stdin().read_to_string(&mut input) {
                eprintln!("Error in input: {}", &e);
                exit(1);
            }

            match Cell::from_json(&input) {
                Ok(cell) => {
                    println!("{}", &cell);
                }
                Err(e) => {
                    eprintln!("JSON decode error: {}", &e);
                    exit(1);
                }
            };

            Ok(())
        }
    }
}

pub(crate) mod book;
pub(crate) mod format;
pub(crate) mod options;
pub(crate) mod plugins;
