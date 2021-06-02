/*
   Copyright 2021 Credera

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#![warn(missing_docs)]

//! kvasir - source file parser and template generator
//!
//! kvasir is a tool for parsing multiple types of source files into JSON format, either
//! outputting directly to stdout or processing the data through one or more templates
//! to generate different output formats.
//!
//! Rather than focus on source code file formats (like Java, Python and Go) for which
//! documentation tools already exist, kvasir is intended to parse and document
//! configuration and human-readable file formats such as YAML, XML, OpenAPI and JSON.
//! With the ability to parse these formats into a single structure, it's possible to
//! generate more flexible output.
//!
//! It can be run directly or within CI/CD pipelines to generate and embed
//! documentation into markdown files, READMEs or other documentation tools.
//!
//! EXAMPLES:
//!```rust
//!     kvasir parse --globs /path/to/**/*.yaml /path/to/**/*.xml
//!     kvasir document --globs /path/to/**/*.yaml --templates templates/base.tpl
//!```

mod errors;
mod parsers;
mod templates;

#[macro_use]
extern crate error_chain;

use env_logger::Env;
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
#[structopt(name = "kvasir", version = "0.1")]
/// kvasir - source file parser and template generator
///
/// kvasir is a tool for parsing multiple types of source files into JSON format, either
/// outputting directly to stdout or processing the data through one or more templates
/// to generate different output formats.
///
/// Rather than focus on source code file formats (like Java, Python and Go) for which
/// documentation tools already exist, kvasir is intended to parse and document
/// configuration and human-readable file formats such as YAML, XML, OpenAPI and JSON.
/// With the ability to parse these formats into a single structure, it's possible to
/// generate more flexible output.
///
/// It can be run directly or within CI/CD pipelines to generate and embed
/// documentation into markdown files, READMEs or other documentation tools.
///
/// EXAMPLES:
///     kvasir parse --globs /path/to/**/*.yaml /path/to/**/*.xml
///     kvasir document --globs /path/to/**/*.yaml --templates templates/base.tpl
struct CLOptions {
    #[structopt(short, long)]
    /// Enable debug application output.
    debug: bool,
    #[structopt(subcommand)]
    /// Subcommand to run.
    cmd: Command,
}

#[derive(Debug, StructOpt)]
/// Command line sub-command to execute
enum Command {
    /// Parse one or more source files into a single JSON structure.
    Parse {
        #[structopt(long)]
        /// One or more glob path expressions to search for source files.
        sources: Vec<String>,
    },

    /// Parse one or more source files into a single JSON structure and format the structure using the
    /// specified templates.
    Document {
        #[structopt(long)]
        /// One or more glob path expressions to search for source files.
        sources: Vec<String>,
        #[structopt(short, long)]
        /// A glob path expression to search for template files
        templates: String,
        #[structopt(short, long)]
        /// Relative path to the base template, if more than one are found by the template glob expression.
        base_template: Option<String>,
        /// Delimiter to search for in the template output to split files.
        #[structopt(short, long)]
        file_split_delimiter: Option<String>,
    },

    /// List available file format parsers.
    Parsers {},
}

/// Initialise the logging environment.
fn logger_environment(verbose: bool) -> Env<'static> {
    env_logger::Env::new()
        .filter_or(
            "KVASIR_LOG",
            if verbose {
                "kvasir=debug"
            } else {
                "kvasir=warn"
            },
        )
        .write_style("KVASIR_LOG_STYLE")
}

/// Application entry point.
fn main() {
    let opts = CLOptions::from_args();

    // Initialise the logger
    env_logger::init_from_env(logger_environment(opts.debug));

    match opts.cmd {
        Command::Parse { sources: globs } => {
            let (successes, failures) = parse_files(globs);
            println!("{}", serde_json::to_string_pretty(&successes).unwrap())
        }
        Command::Document {
            sources: globs,
            templates,
            base_template: base,
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

/// Find the base template to use, based on the number of templates and user choice.
fn get_base_template(
    template_expr: String,
    template_names: &[&str],
    base_template: Option<String>,
) -> Option<String> {
    match template_names {
        [] => {
            error!("No templates found for glob expression: {}", template_expr);
            None
        }
        [single] => Some(single.to_string()),
        [first, ..] => Some(base_template.map_or_else(
            || {
                warn!(
                    "No base template specified. Using first template found: {}",
                    first
                );
                first.to_string()
            },
            |b| b,
        )),
    }
}

/// Parse a list of files using one or more parsers, returning a list of successes and failures.
///
/// Each file is provided to each parser in turn, first to check whether it can be parsed and
/// then to attempt to parse it. Parsing errors are not fatal and do not prevent continuing
/// parsing remaining files.
fn parse_files(globs: Vec<String>) -> (Vec<ParseSuccess>, Vec<ParseFailure>) {
    let (files, errors) = list_files(globs);

    info!("{} files to process.", &files.len());

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

    info!("{} parsers succeeded.", &successes.len());
    info!("{} parsers failed.", &failures.len());
    return (successes, failures);
}

/// Return a list of all unique paths that match one or more glob expressions.
///
/// Paths which appear in more than one glob expression are de-duplicated.
fn list_files(globs: Vec<String>) -> (Vec<PathBuf>, Vec<GlobError>) {
    globs
        .iter()
        .flat_map(|g| glob::glob(g))
        .flatten()
        .unique_by(|r| match r {
            Ok(p) => p.to_owned(),
            Err(e) => e.path().to_path_buf(),
        })
        .partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        })
}

/// Parse a file with one or more file parsers, returning a list of successes and failures.
///
/// Each file is first checked to see whether it *can* be parsed by a given file parser,
/// which is intended to be a computationally and IO-cheap activity. Parsers which indicate
/// that they can parse a file are then called to parse it into a JSON structure.
///
/// File contents are available to both the parsing and parse check code. The contents are
/// retrieved in an efficient fashion so that they are never read more than once. Files
/// are read fully into memory.
fn parse_file(
    f: &PathBuf,
    parsers: &Vec<Box<dyn FileParser>>,
) -> (Vec<ParseSuccess>, Vec<ParseFailure>) {
    info!("{}:", f.display());

    let contents: OnceCell<String> = OnceCell::new();
    let get_contents = || -> Result<&str, Error> {
        let c = contents.get_or_try_init(|| fs::read_to_string(f))?;
        Ok(c.as_str())
    };

    let (parsed, errors): (Vec<ParseSuccess>, Vec<ParseFailure>) = parsers
        .iter()
        .filter(|p| p.can_parse(f, get_contents()))
        .map(|p| p)
        .partition_map(|p| match p.parse(f, get_contents()) {
            Ok(c) => {
                debug!("  succeeded parsing with {}.", p.name());
                Either::Left(ParseSuccess {
                    path: f.to_owned(),
                    parser: p.name().to_owned(),
                    contents: c,
                })
            }
            Err(e) => {
                warn!("  failed parsing with {} ({}).", p.name(), e.to_string());
                Either::Right(ParseFailure {
                    path: f.to_owned(),
                    parser: p.name().to_owned(),
                    error: e,
                })
            }
        });

    (parsed, errors)
}
