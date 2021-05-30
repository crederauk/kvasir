mod errors;
mod parsers;
mod templates;

#[macro_use]
extern crate error_chain;

use errors::Error;
use glob::GlobError;
use itertools::{Either, Itertools};
use log::{debug, error, info, warn};
use once_cell::unsync::OnceCell;
use parsers::FileParser;
use parsers::{ParseFailure, ParseSuccess};
use std::fs;
use std::{path::PathBuf, str};
use structopt::StructOpt;
use tera::Context;

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
        templates: String,
        #[structopt(short, long)]
        base: Option<String>,
        #[structopt(short, long)]
        file_split_delimiter: Option<String>,
    },

    Parsers {},
}

fn main() {
    // Initialise the logger
    env_logger::init();
    println!("Log level: {}", log::max_level());

    let opts = CLOptions::from_args();
    match opts.cmd {
        Command::Parse { globs } => {
            let (successes, failures) = parse_files(globs);
            println!("{}", serde_json::to_string_pretty(&successes).unwrap())
        }
        Command::Document {
            globs,
            templates,
            base,
            file_split_delimiter,
        } => {
            use tera::Tera;
            match Tera::new(templates.as_str()).as_mut() {
                Ok(tera) => {
                    let base_template = get_base_template(
                        templates,
                        tera.get_template_names().collect_vec().as_slice(),
                        base,
                    );

                    // Add filters
                    tera.register_filter("jsonPath", templates::filters::json_path);

                    base_template.map(|template| {
                        let (successes, failures) = parse_files(globs);
                        let mut context = Context::new();
                        context.insert("files", &successes);
                        println!(
                            "{}",
                            tera.render(template.as_str(), &context)
                                .unwrap_or_else(|e| {
                                    error!("Could not render template: {:?}", e);
                                    "".to_string()
                                })
                        )
                    });
                }
                Err(e) => error!("Could not parse templates: {:?}", e),
            }
        }
        Command::Parsers {} => parsers::parsers()
            .iter()
            .for_each(|p| println!("{}", p.name())),
    }
}

fn get_base_template(
    template_expr: String,
    template_names: &[&str],
    base: Option<String>,
) -> Option<String> {
    match template_names {
        [] => {
            error!("No templates found for glob expression: {}", template_expr);
            None
        }
        [single] => Some(single.to_string()),
        [first, ..] => Some(base.map_or_else(
            || {
                warn!(
                    "No base template specified. Using first template: {}",
                    first
                );
                first.to_string()
            },
            |b| b,
        )),
    }
}

fn parse_files(globs: Vec<String>) -> (Vec<ParseSuccess>, Vec<ParseFailure>) {
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
            "{}: failed parsing with {} ({})",
            &f.path.display(),
            &f.parser,
            &f.error.to_string()
        )
    });

    return (successes, failures);
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

    let contents: OnceCell<String> = OnceCell::new();
    let get_contents = || -> Result<&str, Error> {
        let c = contents.get_or_try_init(|| fs::read_to_string(f))?;
        Ok(c.as_str())
    };

    let (parsed, errors): (Vec<ParseSuccess>, Vec<ParseFailure>) = parsers
        .iter()
        .filter(|p| p.can_parse(f, get_contents()))
        .map(|p| {
            debug!("  parseable with {} parser", p.name());
            p
        })
        .partition_map(|p| match p.parse(f, get_contents()) {
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
