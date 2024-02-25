use gumdrop::Options;
use indexmap::IndexMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct LangConfig {
    /// A list of extensions that the language will match against; case-insensitive.
    pub extensions: Vec<String>,

    /// What should count as the starter token for a line comment.
    /// The token itself as well as anything that follows it on that line will be ignored.
    pub line_comments: Vec<String>,

    /// Pairs of multi-line comment openers/closers.
    pub multiline_comments: Vec<(String, String)>,

    /// Pairs of string starters/enders.
    pub strings: Vec<String>,

    /// Tokens which should be excluded from participating in other tokens.
    #[serde(default)]
    pub blacklist: Vec<String>,

    /// Whether or not to keep track of how many multi-line comments were opened; defaults to `false`.
    #[serde(default)]
    pub nested_comments: bool,

    /// Whether or not to keep strings around; controlled by the global config.
    #[serde(skip)]
    pub keep_strings: bool,
}

#[cfg(test)]
impl LangConfig {
    pub fn line_comment(mut self, comment: &str) -> Self {
        self.line_comments.push(comment.to_string());
        self
    }

    pub fn multiline_comment(mut self, start: &str, end: &str) -> Self {
        self.multiline_comments
            .push((start.to_string(), end.to_string()));
        self
    }

    pub fn string(mut self, delimiter: &str) -> Self {
        self.strings.push(delimiter.to_string());
        self
    }

    pub fn blacklist(mut self, token: &str) -> Self {
        self.blacklist.push(token.to_string());
        self
    }

    pub fn nested_comments(mut self, nested: bool) -> Self {
        self.nested_comments = nested;
        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keep_strings: bool,

    #[serde(alias = "lang", default)]
    pub langs: IndexMap<String, LangConfig>,
}

impl Config {
    /// Merges self with other, where the language configs of `other.langs` override those in `self`
    pub fn merge(mut self, other: Config) -> Self {
        self.langs.extend(other.langs.into_iter());

        Self {
            keep_strings: self.keep_strings || other.keep_strings,

            langs: self.langs,
        }
    }
}

#[derive(Options)]
pub struct RuntimeConfig {
    #[options(free)]
    pub filename: Option<String>,

    pub help: bool,

    // TODO: investigate how expensive it would be to just read the file by default.
    #[options(
        help = "When set, the file passed will be read. By default, the file is read from stdin."
    )]
    pub read: bool,

    #[options(
        short = "s",
        help = "When set, strings will be kept in the output file, ignoring the behaviour specified by the config file."
    )]
    pub keep_strings: bool,

    #[options(
        short = "S",
        help = "When set, strings will be removed from the output file, ignoring the behaviour specified by the config file."
    )]
    pub remove_strings: bool,
}
