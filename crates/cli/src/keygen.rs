//! `musubi keygen` — generate a fresh random key.

use std::path::PathBuf;

use clap::Parser;
use musubi_core::{Alphabet, Key};
use rand::rngs::OsRng;
use rand::SeedableRng;

use crate::io::write_output;

/// Arguments for `musubi keygen`.
#[derive(Parser)]
pub struct Args {
    /// Output file. Defaults to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Seed the PRNG with a `u64` for a reproducible key.
    ///
    /// **Do not** use a seeded key for real messages — predictable seeds
    /// produce predictable keys. This flag exists for tests and demos.
    #[arg(long)]
    seed: Option<u64>,
}

/// Run the `keygen` subcommand.
pub fn run(args: &Args) -> anyhow::Result<()> {
    let alphabet = Alphabet::default_v1();
    let key = match args.seed {
        Some(seed) => {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            Key::random(&alphabet, &mut rng)
        }
        None => {
            let mut rng = OsRng;
            Key::random(&alphabet, &mut rng)
        }
    };
    let mut json = key.to_json();
    json.push('\n');
    write_output(args.output.as_deref(), json.as_bytes())
}
