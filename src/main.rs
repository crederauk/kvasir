use glob::GlobError;
use itertools::{Either, Itertools};
use log::{debug, info, warn};
use parsers::FileParser;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::parsers::{ParseFailure, ParseSuccess};

mod parsers;

#[macro_use]
extern crate error_chain;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "kvasir",
    about = "Kvasir automated documentation parser",
    version = "0.1"
)]
struct CLOptions {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    Parse {
        #[structopt(short, long)]
        globs: Vec<String>,
    },

    Document {
        #[structopt(short, long)]
        globs: Vec<String>,
        #[structopt(short, long)]
        templates: Option<Vec<String>>,
    },
}

fn main() {
    // Initialise the logger
    env_logger::init();
    println!("Log level: {}", log::max_level());

    let opts = CLOptions::from_args();
    match opts.cmd {
        Command::Parse { globs } => {
            let (files, errors) = list_files(globs);

            info!("Files to process: {}", &files.len());

            // List errors without exiting
            errors
                .iter()
                .for_each(|e| warn!("Error listing file: {}", e));

            let available_parsers = parsers::parsers();
            let (successes, failures): (Vec<ParseSuccess>, Vec<ParseFailure>) = files
                .iter()
                .map(|f| parse_file(f, &available_parsers))
                .fold((Vec::new(), Vec::new()), |mut last, mut curr| {
                    last.0.append(&mut curr.0);
                    last.1.append(&mut curr.1);
                    (last.0, last.1)
                });

            info!("Successful parsers: {}", &successes.len());
            info!("Failed parsers: {}", &failures.len());
            failures.iter().for_each(|f| {
                warn!(
                    "{}: failed parsing with parser {} ({})",
                    &f.path.display(),
                    &f.parser,
                    &f.error.to_string()
                )
            });

            println!("{}", serde_json::to_string_pretty(&successes).unwrap())
        }
        Command::Document { globs, templates } => println!("Something else was chosen!"),
    }
}

fn list_files(globs: Vec<String>) -> (Vec<PathBuf>, Vec<GlobError>) {
    globs
        .iter()
        .flat_map(|g| glob::glob(g))
        .flatten()
        .partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        })
}

fn parse_file(
    f: &PathBuf,
    parsers: &Vec<Box<dyn FileParser>>,
) -> (Vec<ParseSuccess>, Vec<ParseFailure>) {
    info!("Parsing: {}", f.display());
    let (parsed, errors): (Vec<ParseSuccess>, Vec<ParseFailure>) = parsers
        .iter()
        .filter(|p| p.can_parse(f))
        .map(|p| {
            debug!("  parseable with {} parser", p.name());
            p
        })
        .partition_map(|p| match p.parse(f) {
            Ok(c) => Either::Left(ParseSuccess {
                path: f.to_owned(),
                parser: p.name().to_owned(),
                contents: c,
            }),
            Err(e) => Either::Right(ParseFailure {
                path: f.to_owned(),
                parser: p.name().to_owned(),
                error: e,
            }),
        });

    (parsed, errors)
}
