use std::io::{BufRead, IoSlice, Write};

use crate::*;

/// Half-open range `[start; end)`
#[derive(Clone, Copy, PartialEq, Debug)]
struct Range {
    start: usize,
    end: usize,
}

impl Range {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn contains(&self, index: usize) -> bool {
        self.start <= index && index < self.end
    }

    fn widen(&mut self, start: usize, end: usize) {
        if self.start == self.end {
            self.start = start;
            self.end = end;
            return;
        }

        self.start = self.start.min(start);
        self.end = self.end.max(end);
    }

    fn remove(&self, negative: &mut NegativeRange) {
        if self.start != self.end {
            negative.remove(self.start, self.end);
        }
    }

    /// Returns true if ∃ x, x ∈ self and x ∈ other
    fn overlaps(&self, other: &Self) -> bool {
        if self.start == self.end || other.start == other.end {
            false
        } else {
            self.start + 1 <= other.end && other.start + 1 <= self.end
        }
    }
}

#[derive(Debug)]
struct Ranges {
    open_ranges: Vec<usize>,
    closed_ranges: Vec<Range>,
}

impl Ranges {
    fn empty() -> Self {
        Self {
            open_ranges: Vec::new(),
            closed_ranges: Vec::new(),
        }
    }

    fn contains(&self, index: usize) -> bool {
        self.open_ranges.iter().any(|start| *start <= index)
            || self.closed_ranges.iter().any(|range| range.contains(index))
    }

    /// Gets rid of all the closed ranges, and shifts all the starting indices for the open ranges to zero.
    fn next_line(&mut self) {
        self.closed_ranges.clear();
        for range in &mut self.open_ranges {
            *range = 0;
        }
    }

    fn open(&mut self, start: usize) {
        self.open_ranges.push(start);
    }

    fn close(&mut self, end: usize) {
        if let Some(start) = self.open_ranges.pop() {
            self.closed_ranges.push(Range::new(start, end));
        }
    }

    fn remove(&self, negative: &mut NegativeRange) {
        for closed in self.closed_ranges.iter() {
            closed.remove(negative);
        }
        for start in self.open_ranges.iter() {
            negative.remove(*start, usize::MAX);
        }
    }
}

#[derive(Debug)]
struct NegativeRange {
    ranges: Vec<Range>,
}

impl NegativeRange {
    fn new(line_end: usize) -> Self {
        Self {
            ranges: vec![Range::new(0, line_end)],
        }
    }

    fn remove(&mut self, start: usize, end: usize) {
        let neg_start = start.saturating_sub(1);
        let neg_end = end;

        self.ranges = self
            .ranges
            .drain(..)
            .flat_map(|range| -> [Option<Range>; 2] {
                let mut lhs = None;
                let mut rhs = None;

                if neg_start > range.start {
                    // The input doesn't begin at the lhs of the range:
                    // range : [=====...
                    // input :    [==...
                    // lhs   : [=]
                    lhs = Some(Range::new(range.start, neg_start.min(range.end)));
                }
                if range.end > neg_end {
                    // The input doesn't end at the rhs of the range:
                    // range : ...=====]
                    // input : ...==]
                    // rhs   :       [=]
                    rhs = Some(Range::new(neg_end.max(range.start), range.end));
                }

                [lhs, rhs]
            })
            .flatten()
            .collect();
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum TokenKind {
    LineComment,
    MultiStart(usize),
    MultiEnd(usize),
    String(usize),
}

pub fn handle_input(config: LangConfig, input: impl BufRead, mut output: impl Write) {
    let all_tokens = config
        .multiline_comments
        .iter()
        .enumerate()
        .flat_map(|(index, pair)| {
            [
                (pair.0.clone(), TokenKind::MultiStart(index)),
                (pair.1.clone(), TokenKind::MultiEnd(index)),
            ]
        })
        .chain(
            config
                .strings
                .iter()
                .enumerate()
                .map(|(index, delimiter)| (delimiter.clone(), TokenKind::String(index))),
        )
        .chain(
            config
                .line_comments
                .iter()
                .map(|token| (token.clone(), TokenKind::LineComment)),
        )
        .collect::<Vec<_>>();

    let newline = "\n";
    // A value that gets substituted in in-place of strings
    let string_placeholder = format!(
        "{0}…{0}",
        config.strings.first().cloned().unwrap_or("\"".to_string())
    );

    let mut matches = Vec::with_capacity(64);

    let mut multiline_comments: Vec<usize> = Vec::new();
    let mut multiline_ranges = Ranges::empty();

    let mut current_string: Option<usize> = None;
    let mut string_ranges = Ranges::empty();

    for line in input.lines() {
        multiline_ranges.next_line();
        string_ranges.next_line();
        let line = line.expect("Couldn't read from input!");
        let mut line_range = Range::new(0, 0);

        for (token_string, token_kind) in all_tokens.iter() {
            for (start, match_str) in line.match_indices(token_string) {
                matches.push((Range::new(start, start + match_str.len()), *token_kind));
            }
        }

        for blacklist in collect_blacklist_ranges(&line, &config.blacklist) {
            matches.retain(|(range, _)| !range.overlaps(&blacklist));
        }

        matches.sort_unstable_by_key(|(range, _)| range.start);

        for (range, token_kind) in matches.drain(..) {
            match token_kind {
                TokenKind::LineComment => {
                    if multiline_ranges.contains(range.start) || string_ranges.contains(range.start)
                    {
                        continue;
                    }
                    line_range.widen(range.start, line.len() + 1);
                }
                TokenKind::MultiStart(index) => {
                    if line_range.contains(range.start) || string_ranges.contains(range.start) {
                        continue;
                    }

                    if config.nested_comments {
                        multiline_ranges.open(range.start);
                        multiline_comments.push(index);
                    } else if multiline_ranges.open_ranges.is_empty() {
                        multiline_ranges.open(range.start);
                    }
                }
                TokenKind::MultiEnd(index) => {
                    if line_range.contains(range.start) || string_ranges.contains(range.start) {
                        continue;
                    }

                    // When nested comments are active, verify that the closing comment matches the opening comment;
                    // otherwise, just close the multiline comment, if it is opened.
                    if !config.nested_comments
                        || multiline_comments
                            .last()
                            .map_or(false, |expected| *expected == index)
                    {
                        if config.nested_comments {
                            multiline_comments.pop();
                        }

                        multiline_ranges.close(range.end);
                    }
                }
                TokenKind::String(index) => {
                    if line_range.contains(range.start) || multiline_ranges.contains(range.start) {
                        debug_assert!(current_string.is_none());
                        continue;
                    }

                    match current_string {
                        Some(expected) if expected == index => {
                            current_string = None;
                            string_ranges.close(range.end);
                        }
                        None => {
                            current_string = Some(index);
                            string_ranges.open(range.start);
                        }
                        _ => {}
                    }
                }
            }
        }

        if !config.nested_comments {
            debug_assert!(multiline_comments.len() == 0);
            debug_assert!(multiline_ranges.open_ranges.len() <= 1);
        }

        let mut negative_range = NegativeRange::new(line.len());
        line_range.remove(&mut negative_range);
        multiline_ranges.remove(&mut negative_range);
        if !config.keep_strings {
            string_ranges.remove(&mut negative_range);
        }

        if line.is_empty() {
            output
                .write(newline.as_bytes())
                .expect("Couldn't write to output!");
        } else {
            let mut slices = negative_range
                .ranges
                .into_iter()
                .map(|range| {
                    (
                        IoSlice::new(
                            line[range.start..=range.end.min(line.len().saturating_sub(1))]
                                .as_bytes(),
                        ),
                        range.start,
                    )
                })
                .collect::<Vec<_>>();

            if !config.keep_strings {
                slices = slices
                    .into_iter()
                    .chain(
                        string_ranges
                            .open_ranges
                            .iter()
                            .map(|start| Range::new(*start, line.len() + 1))
                            .chain(string_ranges.closed_ranges.iter().cloned())
                            .map(|range| {
                                (IoSlice::new(string_placeholder.as_bytes()), range.start)
                            }),
                    )
                    .collect();
            }

            slices.sort_unstable_by_key(|pair| pair.1);
            let slices = slices
                .into_iter()
                .map(|pair| pair.0)
                .chain([IoSlice::new(newline.as_bytes())])
                .collect::<Vec<_>>();

            output
                .write_vectored(&slices)
                .expect("Couldn't write to output!");
        }
    }
}

fn collect_blacklist_ranges(line: &str, blacklists: &[String]) -> Vec<Range> {
    let mut ranges = blacklists
        .iter()
        .flat_map(|blacklist| line.match_indices(blacklist))
        .map(|(start, string)| Range::new(start, start + string.len()))
        .collect::<Vec<_>>();

    ranges.sort_unstable_by_key(|range| range.start);

    ranges
        .iter()
        .copied()
        .enumerate()
        .filter(|&(i, range)| !ranges.iter().take(i).any(|other| other.overlaps(&range)))
        .map(|pair| pair.1)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_handle_input(config: LangConfig, input: &'static str, expected: &'static str) {
        let mut output: Vec<u8> = Vec::new();
        handle_input(config, std::io::Cursor::new(input), &mut output);

        let output = String::from_utf8(output).expect("handle_input did not return valid UTF-8");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_overlap() {
        fn test_overlap([xmin, xsup]: [usize; 2], [ymin, ysup]: [usize; 2], positive: bool) {
            assert_eq!(
                Range::new(xmin, xsup).overlaps(&Range::new(ymin, ysup)),
                positive,
                "Range [{xmin}; {xsup}) should{} overlap with [{ymin}; {ysup})",
                if positive { "" } else { "n't" }
            );

            assert_eq!(
                Range::new(ymin, ysup).overlaps(&Range::new(xmin, xsup)),
                positive,
                "Range [{ymin}; {ysup}) should{} overlap with [{xmin}; {xsup})",
                if positive { "" } else { "n't" }
            );
        }

        test_overlap([0, 0], [0, 100], false);
        test_overlap([50, 50], [0, 100], false);
        test_overlap([100, 100], [0, 100], false);
        test_overlap([0, 1], [0, 1], true);
        test_overlap([0, 1], [1, 2], false);
        test_overlap([1, 2], [0, 1], false);
        test_overlap([5, 10], [9, 12], true);
        test_overlap([5, 10], [3, 6], true);
        test_overlap([5, 10], [10, 12], false);
        test_overlap([5, 10], [3, 5], false);
        test_overlap([5, 10], [4, 11], true);
        test_overlap([5, 10], [3, 12], true);
    }

    #[test]
    fn test_noop() {
        test_handle_input(LangConfig::default(), "\n", "\n");

        test_handle_input(LangConfig::default(), "abc", "abc\n");
        test_handle_input(LangConfig::default().nested_comments(true), "abc", "abc\n");

        test_handle_input(LangConfig::default(), "abc\n", "abc\n");
        test_handle_input(
            LangConfig::default().nested_comments(true),
            "abc\n",
            "abc\n",
        );

        test_handle_input(LangConfig::default(), "abc\ndef", "abc\ndef\n");
        test_handle_input(
            LangConfig::default().nested_comments(true),
            "abc\ndef",
            "abc\ndef\n",
        );
    }

    #[test]
    fn test_singleline() {
        let config = LangConfig::default().line_comment("//");

        test_handle_input(config.clone(), "hello // world", "hello \n");
        test_handle_input(
            config.clone(),
            "hello // world\nuntouched",
            "hello \nuntouched\n",
        );
        test_handle_input(config.clone(), "hello // world // hey", "hello \n");
        test_handle_input(config.clone(), "hello // world\n// hey", "hello \n\n");
    }

    #[test]
    fn test_multiline() {
        let config = LangConfig::default().multiline_comment("/*", "*/");

        test_handle_input(config.clone(), "hello /* world", "hello \n");
        test_handle_input(config.clone(), "hello */ world", "hello */ world\n");
        test_handle_input(config.clone(), "hello /* world */", "hello \n");
        test_handle_input(
            config.clone(),
            "hello /* world */\na very long line with a lot of text",
            "hello \na very long line with a lot of text\n",
        );
        test_handle_input(
            config.clone(),
            "hello /* world\n*/newline",
            "hello \nnewline\n",
        );
        test_handle_input(
            config.clone(),
            "hello /* world */ included",
            "hello  included\n",
        );
    }

    #[test]
    fn test_interactions() {
        let config = LangConfig::default()
            .multiline_comment("/*", "*/")
            .line_comment("//");

        // A multiline comment in a single-line comment should be ignored
        test_handle_input(
            config.clone(),
            "hello // /* world\nincluded",
            "hello \nincluded\n",
        );
        test_handle_input(
            config.clone(),
            "hello // /* world\nincluded */",
            "hello \nincluded */\n",
        );

        // A single-line comment in a multiline comment should be ignored
        test_handle_input(config.clone(), "/* // */hello world", "hello world\n");
        test_handle_input(
            config.clone(),
            "/* // */hello world // commented",
            "hello world \n",
        );

        // If a single-line comment is merged with a multiline comment, then the latter does not apply
        test_handle_input(config.clone(), "//* hello */\nworld", "\nworld\n");
        test_handle_input(config.clone(), "/*\n*//world", "\n/world\n");

        let alt_config = LangConfig::default()
            .multiline_comment("/-", "-/")
            .line_comment("--");

        // Equivalently, if a multi-line comment is merged with a single line comment, then the latter does not apply
        test_handle_input(alt_config.clone(), "/-- hello -/\nworld", "\nworld\n");
        test_handle_input(alt_config.clone(), "/-\n--/world", "\nworld\n");
    }

    #[test]
    fn test_nested_comments() {
        let config = LangConfig::default()
            .multiline_comment("/*", "*/")
            .multiline_comment("(-", "-)")
            .nested_comments(true);

        test_handle_input(config.clone(), "/* /* abc */ def */hello", "hello\n");
        test_handle_input(config.clone(), "/* /* abc\n */ def\n*/hello", "\n\nhello\n");

        test_handle_input(config.clone(), "/* (- abc */ def -)hello", "\n");
        test_handle_input(config.clone(), "/* -) abc */ def", " def\n");
    }

    #[test]
    fn test_string() {
        let config = LangConfig::default()
            .multiline_comment("/*", "*/")
            .line_comment("//")
            .string("\"")
            .string("'");

        test_handle_input(config.clone(), "let a = \"hello\";", "let a = \"…\";\n");
        test_handle_input(
            config.clone(),
            "let a = \"hello\";\na very long line with a lot of text",
            "let a = \"…\";\na very long line with a lot of text\n",
        );
        test_handle_input(
            config.clone(),
            "let a = \"hello // world\";",
            "let a = \"…\";\n",
        );
        test_handle_input(
            config.clone(),
            "let a = \"Jack'o'lantern\";",
            "let a = \"…\";\n",
        );
        test_handle_input(config.clone(), "let a = 'hello';", "let a = \"…\";\n");
        test_handle_input(
            config.clone(),
            "let a = 'hello', 'world';",
            "let a = \"…\", \"…\";\n",
        );

        test_handle_input(
            config.clone(),
            "let a = \"hello /* world \";",
            "let a = \"…\";\n",
        );
        test_handle_input(
            config.clone(),
            "let a = \"hello /* world \";\n*/ this is // normal",
            "let a = \"…\";\n*/ this is \n",
        );
    }

    #[test]
    fn test_blacklist() {
        let config = LangConfig::default().string("'").blacklist("\\'");

        test_handle_input(config.clone(), "let a = '\\'';", "let a = '…';\n");
        test_handle_input(
            config.clone(),
            "let a = 'hello \\' world';",
            "let a = '…';\n",
        );

        let config = LangConfig::default()
            .string("'")
            .blacklist("\\\\")
            .blacklist("\\'");

        test_handle_input(config.clone(), "let a = '\\\\\\'';", "let a = '…';\n");
        test_handle_input(config.clone(), "let a = '\\\\';", "let a = '…';\n");
    }
}
