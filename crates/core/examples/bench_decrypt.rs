//! Quick wall-clock benchmark for `decrypt` across a sweep of plaintext
//! lengths and both encoder strategies.
//!
//! Run with:
//!     cargo run --release --example `bench_decrypt`
//!
//! Output is CSV on stdout (`n`, strategy, `encrypt_ms`, `decrypt_ms`) so
//! it can be piped into a plot or table.

use std::time::Instant;

use musubi_core::{decrypt, encrypt, encrypt_chain, Alphabet, Key};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

const SIZES: &[usize] = &[500, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000];
const REPEATS: usize = 5;

fn random_plaintext(alphabet: &Alphabet, n: usize, rng: &mut StdRng) -> String {
    // Build a uniformly random plaintext drawn from the alphabet itself,
    // so we never trip CharOutsideAlphabet.
    let chars: &[char] = alphabet.chars();
    (0..n)
        .map(|_| *chars.choose(rng).expect("alphabet non-empty"))
        .collect()
}

fn main() {
    let alphabet = Alphabet::default_v1();
    let mut rng = StdRng::seed_from_u64(0xBEEF_CAFE);
    let key = Key::random(&alphabet, &mut rng);

    println!("n,strategy,encrypt_ms,decrypt_ms");

    for &n in SIZES {
        let plaintext = random_plaintext(&alphabet, n, &mut rng);

        // ── canonical strategy (v0.1 default, adjacent references) ───────
        let mut enc_ms = 0.0;
        let mut dec_ms = 0.0;
        for _ in 0..REPEATS {
            let t0 = Instant::now();
            let cipher = encrypt(&plaintext, &key, n / 2).unwrap();
            enc_ms += t0.elapsed().as_secs_f64() * 1000.0;

            let t1 = Instant::now();
            let recovered = decrypt(&cipher, &key).unwrap();
            dec_ms += t1.elapsed().as_secs_f64() * 1000.0;

            assert_eq!(recovered.chars().count(), n);
        }
        println!(
            "{n},canonical,{:.3},{:.3}",
            enc_ms / REPEATS as f64,
            dec_ms / REPEATS as f64
        );

        // ── chain strategy (v0.2 random spanning tree) ───────────────────
        let mut enc_ms = 0.0;
        let mut dec_ms = 0.0;
        for _ in 0..REPEATS {
            let mut rng = StdRng::seed_from_u64(7);
            let t0 = Instant::now();
            let cipher = encrypt_chain(&plaintext, &key, n / 2, &mut rng).unwrap();
            enc_ms += t0.elapsed().as_secs_f64() * 1000.0;

            let t1 = Instant::now();
            let recovered = decrypt(&cipher, &key).unwrap();
            dec_ms += t1.elapsed().as_secs_f64() * 1000.0;

            assert_eq!(recovered.chars().count(), n);
        }
        println!(
            "{n},chain,{:.3},{:.3}",
            enc_ms / REPEATS as f64,
            dec_ms / REPEATS as f64
        );
    }
}
