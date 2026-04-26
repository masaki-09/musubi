//! Error types returned by `musubi-core`.

use thiserror::Error;

/// All errors that can occur in the `musubi-core` API.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum MusubiError {
    /// A character was encountered that does not belong to the active alphabet Σ.
    #[error("character {ch:?} is not in the alphabet")]
    CharOutsideAlphabet {
        /// The offending character.
        ch: char,
    },

    /// An [`Alphabet`](crate::Alphabet) was constructed with no characters.
    #[error("alphabet is empty")]
    EmptyAlphabet,

    /// An [`Alphabet`](crate::Alphabet) was constructed with a duplicate character.
    #[error("alphabet contains duplicate character {ch:?}")]
    DuplicateAlphabetChar {
        /// The duplicated character.
        ch: char,
    },

    /// A [`Key`](crate::Key) permutation has the wrong length for its alphabet.
    #[error("permutation length {got} does not match alphabet size {expected}")]
    PermutationLengthMismatch {
        /// Expected length (alphabet size).
        expected: usize,
        /// Actual length supplied.
        got: usize,
    },

    /// A [`Key`](crate::Key) permutation is not a bijection over its alphabet
    /// (contains a duplicate character or a character that is not in the
    /// alphabet).
    #[error("permutation is not a bijection over the alphabet")]
    PermutationNotBijection,

    /// `encrypt` was called with an empty plaintext. musubi requires at least
    /// the anchor character.
    #[error("plaintext is empty")]
    EmptyPlaintext,

    /// The anchor position is out of range for the given plaintext / ciphertext.
    #[error("anchor position {position} is out of range for length {length}")]
    AnchorOutOfRange {
        /// The supplied position.
        position: usize,
        /// The actual length.
        length: usize,
    },

    /// A ciphertext could not be parsed or violates a structural invariant.
    #[error("ciphertext is malformed: {reason}")]
    MalformedCiphertext {
        /// Human-readable description of the violation.
        reason: String,
    },

    /// The ciphertext was produced for a different alphabet than the supplied key.
    #[error("ciphertext alphabet {got:?} does not match expected {expected:?}")]
    AlphabetMismatch {
        /// The alphabet identifier expected (from the key).
        expected: String,
        /// The alphabet identifier found in the ciphertext.
        got: String,
    },
}

/// Crate-local result alias.
pub type Result<T> = std::result::Result<T, MusubiError>;
