mod config;
use config::*;

mod parse;
use parse::*;

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
        // No filename given, default to noop
        noop();
    }

    let extension = args.last().unwrap().split_terminator(".").last().unwrap().to_lowercase();
    for config in configs {
        if config.extensions.iter().find(|ext| **ext == extension).is_some() {
            handle_input(config, std::io::stdin().lock(), std::io::stdout().lock());
            return
        }
    }

    // No language matched, default to noop
    noop();
}

/// If the file does not need to be processed, then we simply pipe it through
fn noop() {
    use std::io::*;

    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    // When ripgrep isn't interested anymore in what we're outputting, it may choose to close the pipe before we're finished writing to it,
    // so we have to handle that case and gracefully shut down:
    match copy(&mut stdin, &mut stdout) {
        Ok(_) => {}
        Err(err) => {
            if err.kind() != ErrorKind::BrokenPipe {
                eprintln!("Error piping stdin to stdout: {:?}", err);
            }
        }
    }

    std::process::exit(0);
}
