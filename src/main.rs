use std::io::{BufRead, BufReader};

use gumdrop::Options;

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
    let runtime_config = RuntimeConfig::parse_args_default_or_exit();

    let Some(input_stream) = get_input_stream(&runtime_config) else {
        eprintln!(
            "No input file specified, or --read-stdin not given. Run `{} --help` for more information.",
            std::env::args().nth(0).unwrap()
        );
        return;
    };

    match get_lang_config(config, &runtime_config) {
        Some(lang_config) => {
            handle_input(lang_config, input_stream, std::io::stdout().lock());
        }
        None => {
            noop(input_stream);
        }
    }
}

/// If the file does not need to be processed, then we simply pipe it through
fn noop(mut input: impl BufRead) {
    use std::io::*;

    let mut stdout = stdout().lock();
    // When ripgrep isn't interested anymore in what we're outputting, it may choose to close the pipe before we're finished writing to it,
    // so we have to handle that case and gracefully shut down:
    match copy(&mut input, &mut stdout) {
        Ok(_) => {}
        Err(err) => {
            if err.kind() != ErrorKind::BrokenPipe {
                eprintln!("Error piping stdin to stdout: {:?}", err);
            }
        }
    }

    std::process::exit(0);
}

#[inline]
fn get_input_stream(runtime_config: &RuntimeConfig) -> Option<Box<dyn BufRead>> {
    if runtime_config.filename.is_some() {
        let filename = runtime_config.filename.clone().unwrap();
        Some(Box::new(BufReader::new(
            std::fs::File::open(filename).expect("Couldn't open specified file"),
        )))
    } else if runtime_config.read_stdin {
        Some(Box::new(std::io::stdin().lock()))
    } else {
        None
    }
}

fn copy_config(lang_config: &mut LangConfig, config: &Config, runtime_config: &RuntimeConfig) {
    if runtime_config.keep_strings {
        lang_config.keep_strings = true;
    } else if runtime_config.remove_strings {
        lang_config.keep_strings = false;
    } else {
        lang_config.keep_strings = config.keep_strings;
    }
}

fn get_lang_config(mut config: Config, runtime_config: &RuntimeConfig) -> Option<LangConfig> {
    if let Some(lang) = &runtime_config.language {
        if let Some(mut lang_config) = config.langs.get(lang).cloned() {
            copy_config(&mut lang_config, &config, &runtime_config);
            return Some(lang_config);
        }
    }

    let Some(filename) = &runtime_config.filename else {
        return None;
    };

    let extension = filename
        .split_terminator(".")
        .last()
        .unwrap()
        .to_lowercase();
    for mut lang_config in std::mem::take(&mut config.langs).into_values().rev() {
        if lang_config.extensions.iter().any(|ext| **ext == extension) {
            copy_config(&mut lang_config, &config, &runtime_config);
            return Some(lang_config);
        }
    }

    None
}
