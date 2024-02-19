# Just the code

A [ripgrep](https://github.com/BurntSushi/ripgrep) preprocessor, that strips away comments and strings from your code,
allowing you to search for *just the code*, which allows you to reduce noise in your search results:

![Example output of just-the-code](./assets/output-example.png)
*Left: original file. Right: output of `just-the-code`.*

Its main advantages over other methods are:

- It is efficient, since it does not need to build a syntax tree or compile your code
- It runs in your terminal and can thus be integrated with other CLI tools
- It can easily be customized to fit your needs: you only a few lines of code to support an additional language
- Accurately handles nested comments, strings and interactions between different kinds of comments
- Integrates natively with `ripgrep`, so using it is as simple as passing one additional parameter

## Installation and usage

*Note: the installation instructions will change once I find the time to publish this app to crates.io*

```sh
# Grab the code
git clone https://github.com/adri326/just-the-code/

# Navigate to the correct repository
cd just-the-code/

# Build the code
cargo build --release

# Install it in your local PATH
cargo install --path .
```

To then use it, simply add `--pre just-the-code` to your ripgrep commands. For instance:

```sh
# Noisy, since a lot of "hello"s are present in strings in the code:
rg "hello"

# No more noise :)
rg --pre just-the-code "hello"
```

*Note: I plan on adding a few options that can be passed to `just-the-code`, so that it can be used outside of ripgrep as well.*

## Adding your own languages

Currently, to add your own language you will need to add it to `src/main.rs` and recompile the code.
I would like to add in the future a simple config file system, so that the code can be compiled once and for all.

The main logic of `just-the-code` has been made generic enough that you only need to tell it how strings and comments
look like for it to work with your language of choice. To do so, you will need to specify the following:

- Single-line comment tokens (`.line_comment("token")`): for instance `//` or `#`; anything after them will be considered part of a comment,
and multiline comments cannot be opened after them.
- Multi-line comment delimiters (`.multiline_comment("start", "end")`): for instance `/*` and `*/`;
single-line comments between them will be ignored.
- String delimiters (`.string("delimiter")`): commenting tokens in strings will be ignored, and string delimiters will be ignored in comments.
- Blacklist tokens (`.blacklist("token")`): any of the tokens specified will **not** be matched if it overlaps with a blacklisted token.
This lets you blacklist `\"` in strings, for instance.
- Whether or not to allow nested comments (`.nested_comments(allow)`):
if enabled, then `a /* /* */ */ b` will become `a  b`; if disabled, that same piece of code will instead become `a  */ b`.

## Known issues

Currently, when given as blacklist for string separators `\"`,
`just-the-code` incorrectly misinterprets `"\\"` as a string opening token, followed by some string contents and the blacklisted `\"`,
so the string won't be considered closed after that.

How to fix this issue in a generic way is tricky.
I am planning on experimenting with mutually exclusive blacklist expressions:
by specifying that `\\` and `\"` are blacklisted tokens, and by ensuring that no two blacklist tokens can overlap,
the string `"\\"` should be handled correctly.

The performance of `ripgrep` also severaly drops when adding `--pre`, since ripgrep essentially needs to `fork()` once for each file searched.
It might be possible in the future to integrate `just-the-code` directly within `ripgrep`, so that everything can be done within the same process.
