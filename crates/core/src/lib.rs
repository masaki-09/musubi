//! musubi-core — relational cipher core library.
//!
//! musubi (結び) is a classical cipher built on a novel theory: instead of
//! substituting characters or transposing them, the ciphertext describes
//! *relationships between* characters. With one anchor character revealed,
//! the receiver can reconstruct the whole message by unwinding the chain.
//!
//! See [`docs/SPEC.md`] for the formal specification and [`docs/THEORY.md`]
//! for the design rationale.
//!
//! [`docs/SPEC.md`]: https://github.com/masaki-09/musubi/blob/main/docs/SPEC.md
//! [`docs/THEORY.md`]: https://github.com/masaki-09/musubi/blob/main/docs/THEORY.md
//!
//! ## Toy cipher disclaimer
//!
//! musubi is **not for serious security**. It is easily broken by classical
//! cryptanalysis. Use it for love letters, puzzles, and the joy of inventing
//! a new classical cipher.
//!
//! ## Quick example
//!
//! ```
//! use musubi_core::{Alphabet, Key, encrypt, decrypt};
//! use rand::SeedableRng;
//!
//! let alphabet = Alphabet::default_v1();
//! let mut rng = rand::rngs::StdRng::seed_from_u64(42);
//! let key = Key::random(&alphabet, &mut rng);
//!
//! let plaintext = "あいしてる";
//! let cipher = encrypt(plaintext, &key, 2).unwrap();
//! let decrypted = decrypt(&cipher, &key).unwrap();
//! assert_eq!(decrypted, plaintext);
//! ```

#![doc(html_root_url = "https://docs.rs/musubi-core")]

/// Crate version string, baked in at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod alphabet;
pub mod cipher;
pub mod error;
pub mod key;
pub mod relation;

pub use alphabet::Alphabet;
pub use cipher::{
    decrypt, encrypt, encrypt_chain, encrypt_woven, Anchor, Ciphertext, CiphertextExt,
    FORMAT_VERSION,
};
pub use error::{MusubiError, Result};
pub use key::Key;
pub use relation::Relation;
