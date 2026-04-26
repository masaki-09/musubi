//! musubi command-line interface.
//!
//! Three subcommands:
//! - `keygen`  — generate a random key for the default alphabet
//! - `encrypt` — turn plaintext into a JSON ciphertext
//! - `decrypt` — recover plaintext from a JSON ciphertext
//!
//! All commands accept `-i/--input` for an input path (stdin if omitted)
//! and `-o/--output` for an output path (stdout if omitted), so they
//! compose naturally in shell pipelines.

mod decrypt;
mod encrypt;
mod io;
mod keygen;

use clap::{Parser, Subcommand};

/// musubi (結び) — relational classical cipher.
///
/// Alphabet: `default-v1` (175 chars). See <https://github.com/masaki-09/musubi>.
#[derive(Parser)]
#[command(name = "musubi", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new key.
    Keygen(keygen::Args),
    /// Encrypt plaintext into a JSON ciphertext.
    Encrypt(encrypt::Args),
    /// Decrypt a JSON ciphertext back into plaintext.
    Decrypt(decrypt::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Keygen(args) => keygen::run(&args),
        Command::Encrypt(args) => encrypt::run(&args),
        Command::Decrypt(args) => decrypt::run(&args),
    }
}
