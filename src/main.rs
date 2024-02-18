mod config;
use config::*;

mod parse;
use parse::*;

use std::io::{BufRead, Write};

fn main() {
    let configs: Vec<LangConfig> = vec![
        LangConfig::default()
            .extension("lean")
            .line_comment("--")
            .line_comment("#align")
            .multiline_comment("/-", "-/")
            .string("\"")
            .blacklist("\\\"")
            .nested_comments(true),
        LangConfig::default()
            .extension("rs")
            .extension("js")
            .extension("ts")
            .extension("c")
            .line_comment("//")
            .multiline_comment("/*", "*/")
            .string("\"")
            .string("'")
            .blacklist("\\'")
            .blacklist("\\\"")
    ];

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() <= 1 || args.last().map_or(false, |arg| !arg.contains('.')) {
        noop();
    }

    let extension = args.last().unwrap().split_terminator(".").last().unwrap().to_lowercase();
    for config in configs {
        if config.extensions.iter().find(|ext| **ext == extension).is_some() {
            handle_input(config, std::io::stdin().lock(), std::io::stdout().lock());
            break
        }
    }
}

/// If the file does not need to be processed, then we simply pipe it through
fn noop() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();

    for line in stdin.lock().lines() {
        stdout.write_all(line.expect("Could not read from stdin").as_bytes()).expect("Could not write to stdout");
    }

    std::process::exit(0);
}
