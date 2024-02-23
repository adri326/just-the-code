mod config;
use config::*;

mod parse;
use parse::*;

fn load_config() -> Config {
    let default_config: Config = toml::from_str(include_str!("./default_config.toml"))
        .expect("Error parsing default config");

    let Some(custom_config) =
        directories::ProjectDirs::from("xyz", "Shad Amethyst", "just-the-code")
    else {
        return default_config;
    };
    let mut custom_config = custom_config.config_dir().to_path_buf();
    custom_config.push("config.toml");

    let Ok(custom_config) = std::fs::read_to_string(custom_config) else {
        return default_config;
    };

    let custom_config = match toml::from_str(&custom_config) {
        Ok(config) => config,
        Err(error) => {
            panic!("Error parsing custom config: {}", error);
        }
    };

    default_config.merge(custom_config)
}

fn main() {
    let config = load_config();

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() <= 1 || args.last().map_or(false, |arg| !arg.contains('.')) {
        // No filename given, default to noop
        noop();
    }

    let extension = args
        .last()
        .unwrap()
        .split_terminator(".")
        .last()
        .unwrap()
        .to_lowercase();
    for mut lang_config in config.langs.into_values().rev() {
        if lang_config
            .extensions
            .iter()
            .find(|ext| **ext == extension)
            .is_some()
        {
            lang_config.keep_strings = config.keep_strings;
            handle_input(
                lang_config,
                std::io::stdin().lock(),
                std::io::stdout().lock(),
            );
            return;
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
