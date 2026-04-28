//! `musubi encrypt` — turn plaintext into a JSON ciphertext.

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use musubi_core::{encrypt, encrypt_woven, Alphabet, Ciphertext};
use rand::rngs::OsRng;
use rand::SeedableRng;

use crate::io::{read_input, read_key, trim_trailing_newline, write_output};

/// Encoder strategy selectable on the command line.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Strategy {
    /// v0.1 canonical encoder — every relation references the immediately
    /// adjacent position toward the anchor. Deterministic, no RNG used.
    Canonical,
    /// v0.2 chain encoder (多重結び) — relations form a uniformly random
    /// spanning tree rooted at the anchor. Required when `--noise > 0`.
    Chain,
}

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

    /// Encoder strategy. `canonical` (default) preserves v0.1 output;
    /// `chain` produces a random spanning tree (and is required for noise).
    #[arg(long, value_enum, default_value_t = Strategy::Canonical)]
    strategy: Strategy,

    /// Inject `noise` dummy characters (迷い糸). Hides the true plaintext
    /// length and structure. Implies `--strategy chain`.
    #[arg(long, default_value_t = 0)]
    noise: usize,

    /// Seed the chain/noise RNG with a `u64` for reproducible output.
    /// Ignored under `--strategy canonical` with `--noise 0`.
    ///
    /// **Do not** use a seeded RNG for real messages.
    #[arg(long)]
    seed: Option<u64>,
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

    let needs_chain = matches!(args.strategy, Strategy::Chain) || args.noise > 0;

    let cipher: Ciphertext = if needs_chain {
        if let Some(seed) = args.seed {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            encrypt_woven(plaintext, &key, anchor, args.noise, &mut rng)
        } else {
            let mut rng = OsRng;
            encrypt_woven(plaintext, &key, anchor, args.noise, &mut rng)
        }
        .with_context(|| format!("failed to encrypt with anchor at position {anchor}"))?
    } else {
        encrypt(plaintext, &key, anchor)
            .with_context(|| format!("failed to encrypt with anchor at position {anchor}"))?
    };

    let mut json = if args.compact {
        serde_json::to_string(&cipher)?
    } else {
        serde_json::to_string_pretty(&cipher)?
    };
    json.push('\n');
    write_output(args.output.as_deref(), json.as_bytes())
}
