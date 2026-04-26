//! I/O helpers shared by the subcommands.

use std::io::{Read, Write};
use std::path::Path;

use anyhow::Context;
use musubi_core::{Alphabet, Key};

/// Read input from a file, or from stdin if `path` is `None`.
pub fn read_input(path: Option<&Path>) -> anyhow::Result<String> {
    match path {
        Some(p) => {
            std::fs::read_to_string(p).with_context(|| format!("failed to read {}", p.display()))
        }
        None => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .context("failed to read stdin")?;
            Ok(s)
        }
    }
}

/// Write `bytes` to a file, or to stdout if `path` is `None`.
pub fn write_output(path: Option<&Path>, bytes: &[u8]) -> anyhow::Result<()> {
    match path {
        Some(p) => {
            std::fs::write(p, bytes).with_context(|| format!("failed to write {}", p.display()))
        }
        None => std::io::stdout()
            .write_all(bytes)
            .context("failed to write stdout"),
    }
}

/// Read a key file and parse it for the given alphabet.
pub fn read_key(path: &Path, alphabet: &Alphabet) -> anyhow::Result<Key> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read key file {}", path.display()))?;
    Key::from_json(&json, alphabet)
        .with_context(|| format!("failed to parse key file {}", path.display()))
}

/// Strip a single trailing `\n` (or `\r\n`) from `s` if present.
///
/// Most shells append a newline when reading a heredoc or `echo` output;
/// trimming exactly one keeps the typical `echo "…" | musubi encrypt`
/// pipeline working without surprising the user with a `\n is not in
/// the alphabet` error.
#[must_use]
pub fn trim_trailing_newline(s: &str) -> &str {
    s.strip_suffix('\n')
        .map(|stripped| stripped.strip_suffix('\r').unwrap_or(stripped))
        .unwrap_or(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_trailing_newline_handles_both_styles() {
        assert_eq!(trim_trailing_newline("hi\n"), "hi");
        assert_eq!(trim_trailing_newline("hi\r\n"), "hi");
        assert_eq!(trim_trailing_newline("hi"), "hi");
        assert_eq!(trim_trailing_newline(""), "");
        // Only a single trailing newline is stripped.
        assert_eq!(trim_trailing_newline("hi\n\n"), "hi\n");
    }
}
