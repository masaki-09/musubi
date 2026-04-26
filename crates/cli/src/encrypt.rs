//! `musubi encrypt` — turn plaintext into a JSON ciphertext.

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use musubi_core::{encrypt, Alphabet};

use crate::io::{read_input, read_key, trim_trailing_newline, write_output};

/// Arguments for `musubi encrypt`.
#[derive(Parser)]
pub struct Args {
    /// Path to a key file produced by `musubi keygen`.
    #[arg(short, long)]
    key: PathBuf,

    /// Plaintext input file. Defaults to stdin.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file. Defaults to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Anchor position (0-indexed). Defaults to the middle of the plaintext.
    #[arg(short, long)]
    anchor: Option<usize>,

    /// Emit compact JSON instead of pretty-printed.
    #[arg(long)]
    compact: bool,
}

/// Run the `encrypt` subcommand.
pub fn run(args: &Args) -> anyhow::Result<()> {
    let alphabet = Alphabet::default_v1();
    let key = read_key(&args.key, &alphabet)?;
    let raw = read_input(args.input.as_deref())?;
    let plaintext = trim_trailing_newline(&raw);
    let n = plaintext.chars().count();
    if n == 0 {
        anyhow::bail!("plaintext is empty");
    }
    let anchor = args.anchor.unwrap_or(n / 2);
    let cipher = encrypt(plaintext, &key, anchor)
        .with_context(|| format!("failed to encrypt with anchor at position {anchor}"))?;
    let mut json = if args.compact {
        serde_json::to_string(&cipher)?
    } else {
        serde_json::to_string_pretty(&cipher)?
    };
    json.push('\n');
    write_output(args.output.as_deref(), json.as_bytes())
}
