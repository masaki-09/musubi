//! WebAssembly bindings for `musubi`.
//!
//! Exposes three JS-callable functions that mirror the CLI subcommands:
//! [`keygen`], [`js_encrypt`] (renamed `encrypt` in JS), and
//! [`js_decrypt`] (renamed `decrypt` in JS).
//!
//! All I/O is via JSON strings to keep the JS interface small and
//! version-stable; the JSON formats match `musubi-core`'s on-disk
//! representations.

#![doc(html_root_url = "https://docs.rs/musubi-wasm")]

use musubi_core::{decrypt, encrypt, encrypt_woven, Alphabet, Ciphertext, Key};
use rand::rngs::OsRng;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;

/// Crate version, baked in at compile time.
pub use musubi_core::VERSION;

fn to_js_error(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

/// Generate a fresh random key for the default-v1 alphabet.
///
/// Uses [`rand::rngs::OsRng`], which is backed by
/// `crypto.getRandomValues` in the browser.
///
/// Returned as a JSON string in the same format that the CLI
/// `musubi keygen` produces.
#[wasm_bindgen]
#[must_use]
pub fn keygen() -> String {
    let alphabet = Alphabet::default_v1();
    let mut rng = OsRng;
    let key = Key::random(&alphabet, &mut rng);
    key.to_json()
}

/// Encrypt `plaintext` with the given key (JSON), revealing the
/// character at `anchor` (defaults to the middle of the message).
///
/// Returned as a pretty-printed JSON ciphertext string.
///
/// # Errors
///
/// Returns a `JsError` if the key cannot be parsed, the plaintext is
/// empty, the anchor is out of range, or any character of the
/// plaintext is not in the alphabet.
#[wasm_bindgen(js_name = encrypt)]
pub fn js_encrypt(
    plaintext: &str,
    key_json: &str,
    anchor: Option<usize>,
) -> Result<String, JsError> {
    let alphabet = Alphabet::default_v1();
    let key = Key::from_json(key_json, &alphabet).map_err(to_js_error)?;
    let n = plaintext.chars().count();
    if n == 0 {
        return Err(JsError::new("plaintext is empty"));
    }
    let pos = anchor.unwrap_or(n / 2);
    let cipher = encrypt(plaintext, &key, pos).map_err(to_js_error)?;
    serde_json::to_string_pretty(&cipher).map_err(to_js_error)
}

/// Encrypt `plaintext` with the v0.2 woven encoder — a random spanning
/// tree rooted at the anchor, optionally interleaved with `noise` dummy
/// characters (迷い糸).
///
/// `noise = 0` produces a chain ciphertext (multi-knot encoder, 多重結び).
/// `noise > 0` produces a noise-injected ciphertext that hides the true
/// plaintext length from anyone without the key.
///
/// `seed` is optional. When provided, the chain/noise RNG is seeded for
/// reproducible output (intended for tests/demos only). When omitted,
/// the browser's `crypto.getRandomValues` (via [`OsRng`]) is used.
///
/// # Errors
///
/// Same as [`js_encrypt`].
#[wasm_bindgen(js_name = encryptWoven)]
pub fn js_encrypt_woven(
    plaintext: &str,
    key_json: &str,
    anchor: Option<usize>,
    noise: Option<usize>,
    seed: Option<u64>,
) -> Result<String, JsError> {
    let alphabet = Alphabet::default_v1();
    let key = Key::from_json(key_json, &alphabet).map_err(to_js_error)?;
    let n = plaintext.chars().count();
    if n == 0 {
        return Err(JsError::new("plaintext is empty"));
    }
    let pos = anchor.unwrap_or(n / 2);
    let noise = noise.unwrap_or(0);
    let cipher = if let Some(s) = seed {
        let mut rng = rand::rngs::StdRng::seed_from_u64(s);
        encrypt_woven(plaintext, &key, pos, noise, &mut rng)
    } else {
        let mut rng = OsRng;
        encrypt_woven(plaintext, &key, pos, noise, &mut rng)
    }
    .map_err(to_js_error)?;
    serde_json::to_string_pretty(&cipher).map_err(to_js_error)
}

/// Decrypt a JSON ciphertext with the given key (JSON), returning the
/// recovered plaintext.
///
/// # Errors
///
/// Returns a `JsError` if either JSON cannot be parsed or the
/// ciphertext fails any of the structural checks in
/// [`musubi_core::decrypt`].
#[wasm_bindgen(js_name = decrypt)]
pub fn js_decrypt(ciphertext_json: &str, key_json: &str) -> Result<String, JsError> {
    let alphabet = Alphabet::default_v1();
    let key = Key::from_json(key_json, &alphabet).map_err(to_js_error)?;
    let cipher: Ciphertext = serde_json::from_str(ciphertext_json).map_err(to_js_error)?;
    decrypt(&cipher, &key).map_err(to_js_error)
}
