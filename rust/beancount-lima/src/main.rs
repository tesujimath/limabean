use crate::{
    digest::Digest,
    format::edn::write_digest_as_edn,
    ingest::{write_ingest_as_json, Ingest},
};
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
    /// Extensible comma-separated options, each split on `=` and passed as strings, or as bools if no `=`
    #[clap(short, value_delimiter = ',')]
    options: Vec<String>,

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

    /// Digest the Beancount file for import
    Digest {
        /// Beancount file path
        beanpath: PathBuf,
    },

    /// Ingest from external CSV or OFX files into EDN
    Ingest {
        /// File to ingest
        ingest_file: PathBuf,
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
        Command::Book {
            beanpath: ledger,
            format,
        } => book::write_bookings_from(
            ledger,
            format.unwrap_or(Format::default()).into(),
            out_w,
            error_w,
        ),

        Command::Digest { beanpath: ledger } => {
            let digest = Digest::load_from(
                ledger,
                ACCID_KEY.to_string(),
                vec![TXNID_KEY.to_string(), TXNID2_KEY.to_string()],
                PAYEE2_KEY.to_string(),
                NARRATION2_KEY.to_string(),
                error_w,
            )?;
            write_digest_as_edn(&digest, out_w)
        }

        Command::Ingest { ingest_file } => {
            let ingest = Ingest::parse_from(ingest_file, error_w)?;
            write_ingest_as_json(&ingest, out_w)
        }

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

const ACCID_KEY: &str = "accid";
const TXNID_KEY: &str = "txnid";
const TXNID2_KEY: &str = "txnid2";
const PAYEE2_KEY: &str = "payee2";
const NARRATION2_KEY: &str = "narration2";

pub(crate) mod book;
pub(crate) mod digest;
pub(crate) mod format;
pub(crate) mod ingest;
pub(crate) mod options;
pub(crate) mod plugins;
