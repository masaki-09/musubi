//! Secret key — the permutation π over the alphabet.
//!
//! A [`Key`] is a permutation of the alphabet that determines each
//! character's *rank*. Encryption and decryption both consult ranks; without
//! the key, an attacker sees only relative offsets in some unknown ordering.

use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::alphabet::Alphabet;
use crate::error::{MusubiError, Result};

/// A secret permutation over the characters of an [`Alphabet`].
///
/// The same alphabet has `|Σ|!` distinct keys. For the default alphabet
/// (`|Σ| = 175`), this is approximately 5.4 × 10³¹⁹ keys.
///
/// Keys serialize to a small JSON document binding the alphabet identifier
/// to the permuted character list:
///
/// ```json
/// { "alphabet": "default-v1", "permutation": ["☃", "あ", ...] }
/// ```
#[derive(Debug, Clone)]
pub struct Key {
    permutation: Vec<char>,
    alphabet_id: String,
    rank_of: HashMap<char, usize>,
}

impl Key {
    /// Construct a key from a permutation of the alphabet's characters.
    ///
    /// The permutation must contain every character of `alphabet` exactly once.
    pub fn new(alphabet: &Alphabet, permutation: Vec<char>) -> Result<Self> {
        if permutation.len() != alphabet.len() {
            return Err(MusubiError::PermutationLengthMismatch {
                expected: alphabet.len(),
                got: permutation.len(),
            });
        }
        let mut rank_of: HashMap<char, usize> = HashMap::with_capacity(permutation.len());
        for (rank, &ch) in permutation.iter().enumerate() {
            if !alphabet.contains(ch) {
                return Err(MusubiError::CharOutsideAlphabet { ch });
            }
            if rank_of.insert(ch, rank).is_some() {
                return Err(MusubiError::PermutationNotBijection);
            }
        }
        Ok(Self {
            permutation,
            alphabet_id: alphabet.id().to_string(),
            rank_of,
        })
    }

    /// Generate a uniformly random key for the given alphabet using `rng`.
    pub fn random<R: Rng + ?Sized>(alphabet: &Alphabet, rng: &mut R) -> Self {
        let mut perm = alphabet.chars().to_vec();
        perm.shuffle(rng);
        Self::new(alphabet, perm).expect("shuffle preserves the alphabet's char set")
    }

    /// Identifier of the alphabet this key belongs to.
    #[must_use]
    pub fn alphabet_id(&self) -> &str {
        &self.alphabet_id
    }

    /// Number of characters in the underlying alphabet.
    #[must_use]
    pub fn len(&self) -> usize {
        self.permutation.len()
    }

    /// Returns `false`; alphabets are non-empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// The character at the given rank, or `None` if `rank` is out of range.
    #[must_use]
    pub fn char_at(&self, rank: usize) -> Option<char> {
        self.permutation.get(rank).copied()
    }

    /// The rank of the given character, or `None` if it is not in the key.
    #[must_use]
    pub fn rank_of(&self, c: char) -> Option<usize> {
        self.rank_of.get(&c).copied()
    }

    /// Serialize the key to a JSON string.
    ///
    /// The output binds the alphabet identifier to the permutation so that
    /// [`Key::from_json`] can reject mismatched alphabets at parse time.
    #[must_use]
    pub fn to_json(&self) -> String {
        let s = KeySerde {
            alphabet: self.alphabet_id.clone(),
            permutation: self.permutation.clone(),
        };
        serde_json::to_string_pretty(&s).expect("KeySerde always serializes")
    }

    /// Parse a key from JSON, validating it against the supplied alphabet.
    pub fn from_json(input: &str, alphabet: &Alphabet) -> Result<Self> {
        let parsed: KeySerde =
            serde_json::from_str(input).map_err(|e| MusubiError::MalformedCiphertext {
                reason: format!("key JSON parse error: {e}"),
            })?;
        if parsed.alphabet != alphabet.id() {
            return Err(MusubiError::AlphabetMismatch {
                expected: alphabet.id().to_string(),
                got: parsed.alphabet,
            });
        }
        Self::new(alphabet, parsed.permutation)
    }
}

#[derive(Serialize, Deserialize)]
struct KeySerde {
    alphabet: String,
    permutation: Vec<char>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn random_key_is_a_permutation() {
        let alphabet = Alphabet::default_v1();
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xCAFE_F00D);
        let key = Key::random(&alphabet, &mut rng);
        assert_eq!(key.len(), alphabet.len());
        for &c in alphabet.chars() {
            assert!(key.rank_of(c).is_some(), "missing {c:?}");
        }
    }

    #[test]
    fn ranks_round_trip() {
        let alphabet = Alphabet::default_v1();
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let key = Key::random(&alphabet, &mut rng);
        for rank in 0..key.len() {
            let ch = key.char_at(rank).unwrap();
            assert_eq!(key.rank_of(ch), Some(rank));
        }
    }

    #[test]
    fn json_round_trips() {
        let alphabet = Alphabet::default_v1();
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);
        let key = Key::random(&alphabet, &mut rng);
        let json = key.to_json();
        let parsed = Key::from_json(&json, &alphabet).unwrap();
        assert_eq!(parsed.len(), key.len());
        for rank in 0..key.len() {
            assert_eq!(parsed.char_at(rank), key.char_at(rank));
        }
    }

    #[test]
    fn wrong_length_errors() {
        let alphabet = Alphabet::default_v1();
        let perm = vec!['a', 'b'];
        assert!(matches!(
            Key::new(&alphabet, perm),
            Err(MusubiError::PermutationLengthMismatch { .. })
        ));
    }

    #[test]
    fn duplicate_in_permutation_errors() {
        let alphabet = Alphabet::new("xyz", vec!['a', 'b', 'c']).unwrap();
        let perm = vec!['a', 'a', 'b'];
        assert!(matches!(
            Key::new(&alphabet, perm),
            Err(MusubiError::PermutationNotBijection)
        ));
    }

    #[test]
    fn alphabet_mismatch_on_parse() {
        let alphabet_a = Alphabet::new("xyz", vec!['a', 'b', 'c']).unwrap();
        let alphabet_b = Alphabet::new("uvw", vec!['a', 'b', 'c']).unwrap();
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let key = Key::random(&alphabet_a, &mut rng);
        let json = key.to_json();
        assert!(matches!(
            Key::from_json(&json, &alphabet_b),
            Err(MusubiError::AlphabetMismatch { .. })
        ));
    }
}
