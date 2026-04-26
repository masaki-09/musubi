//! `musubi decrypt` — recover plaintext from a JSON ciphertext.

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use musubi_core::{decrypt, Alphabet, Ciphertext};

use crate::io::{read_input, read_key, write_output};

/// Arguments for `musubi decrypt`.
#[derive(Parser)]
pub struct Args {
    /// Path to a key file produced by `musubi keygen`.
    #[arg(short, long)]
    key: PathBuf,

    /// Ciphertext JSON input file. Defaults to stdin.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file. Defaults to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Run the `decrypt` subcommand.
pub fn run(args: &Args) -> anyhow::Result<()> {
    let alphabet = Alphabet::default_v1();
    let key = read_key(&args.key, &alphabet)?;
    let raw = read_input(args.input.as_deref())?;
    let cipher: Ciphertext =
        serde_json::from_str(&raw).context("failed to parse ciphertext as JSON")?;
    let mut plaintext = decrypt(&cipher, &key)?;
    plaintext.push('\n');
    write_output(args.output.as_deref(), plaintext.as_bytes())
}
