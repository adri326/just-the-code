
#[derive(Default, Clone, Debug)]
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

    // TODO: use regexes?
    /// Tokens which should be excluded from participating in other tokens.
    pub blacklist: Vec<String>,

    /// Whether or not to keep track of how many multi-line comments were opened; defaults to `false`.
    pub nested_comments: bool,
}

impl LangConfig {
    pub fn extension(mut self, extension: &str) -> Self {
        self.extensions.push(extension.to_string());
        self
    }

    pub fn line_comment(mut self, comment: &str) -> Self {
        self.line_comments.push(comment.to_string());
        self
    }

    pub fn multiline_comment(mut self, start: &str, end: &str) -> Self {
        self.multiline_comments.push((start.to_string(), end.to_string()));
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
