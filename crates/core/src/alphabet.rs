//! The character set Σ that musubi operates over.
//!
//! An [`Alphabet`] is the *unordered* set of characters that musubi can
//! encrypt. Ordering of those characters into a permutation π is the role of
//! [`Key`](crate::Key); the alphabet alone is shared, public, and identified
//! by a stable string ID.

use std::collections::HashSet;

use crate::error::{MusubiError, Result};

/// The default alphabet identifier used by musubi v0.1.
pub const DEFAULT_V1_ID: &str = "default-v1";

const HIRAGANA_SEION: &str =
    "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
const HIRAGANA_DAKUON: &str = "がぎぐげござじずぜぞだぢづでどばびぶべぼ";
const HIRAGANA_HANDAKUON: &str = "ぱぴぷぺぽ";
const HIRAGANA_KOGAKI: &str = "ぁぃぅぇぉっゃゅょ";

/// A finite set of characters that the cipher can operate on.
///
/// An `Alphabet` is identified by a string [`id`](Self::id) so that
/// ciphertexts can record *which* alphabet they were produced for and
/// reject keys built for a different one.
#[derive(Debug, Clone)]
pub struct Alphabet {
    chars: Vec<char>,
    id: String,
}

impl Alphabet {
    /// Construct an alphabet from an ordered list of characters.
    ///
    /// The order of `chars` is preserved as the *canonical* ordering of the
    /// alphabet. The list must be non-empty and contain no duplicates.
    pub fn new(id: impl Into<String>, chars: Vec<char>) -> Result<Self> {
        if chars.is_empty() {
            return Err(MusubiError::EmptyAlphabet);
        }
        let mut seen = HashSet::with_capacity(chars.len());
        for &c in &chars {
            if !seen.insert(c) {
                return Err(MusubiError::DuplicateAlphabetChar { ch: c });
            }
        }
        Ok(Self {
            chars,
            id: id.into(),
        })
    }

    /// The default alphabet for musubi v0.1.
    ///
    /// Contains 175 characters in this canonical order:
    /// 1. 五十音清音 (46): あ-ん
    /// 2. 濁音 (20): が-ぼ
    /// 3. 半濁音 (5): ぱ-ぽ
    /// 4. 小書き仮名 (9): ぁぃぅぇぉっゃゅょ
    /// 5. ASCII printable (95): U+0020 (space) through U+007E (`~`)
    #[must_use]
    pub fn default_v1() -> Self {
        let mut chars: Vec<char> = Vec::with_capacity(175);
        chars.extend(HIRAGANA_SEION.chars());
        chars.extend(HIRAGANA_DAKUON.chars());
        chars.extend(HIRAGANA_HANDAKUON.chars());
        chars.extend(HIRAGANA_KOGAKI.chars());
        chars.extend((0x20u32..=0x7Eu32).filter_map(char::from_u32));
        // Construction is internally controlled, so the invariants hold.
        Self::new(DEFAULT_V1_ID, chars).expect("default-v1 alphabet is well-formed")
    }

    /// Stable identifier used for serialization compatibility checks.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Number of characters in the alphabet.
    #[must_use]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Always `false`; preserved to satisfy `clippy::len_without_is_empty`.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// The canonical ordered list of characters.
    #[must_use]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Whether the alphabet contains the given character.
    #[must_use]
    pub fn contains(&self, c: char) -> bool {
        self.chars.contains(&c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_v1_has_expected_size() {
        let a = Alphabet::default_v1();
        assert_eq!(a.len(), 46 + 20 + 5 + 9 + 95);
        assert_eq!(a.id(), DEFAULT_V1_ID);
    }

    #[test]
    fn default_v1_has_no_duplicates() {
        let a = Alphabet::default_v1();
        let mut seen = HashSet::new();
        for &c in a.chars() {
            assert!(seen.insert(c), "duplicate char {c:?}");
        }
    }

    #[test]
    fn default_v1_contains_expected_chars() {
        let a = Alphabet::default_v1();
        assert!(a.contains('あ'));
        assert!(a.contains('が'));
        assert!(a.contains('ぱ'));
        assert!(a.contains('ゃ'));
        assert!(a.contains('A'));
        assert!(a.contains('0'));
        assert!(a.contains(' '));
        assert!(!a.contains('、'));
        assert!(!a.contains('世'));
    }

    #[test]
    fn empty_alphabet_errors() {
        assert!(matches!(
            Alphabet::new("x", vec![]),
            Err(MusubiError::EmptyAlphabet)
        ));
    }

    #[test]
    fn duplicate_chars_error() {
        assert!(matches!(
            Alphabet::new("x", vec!['a', 'b', 'a']),
            Err(MusubiError::DuplicateAlphabetChar { ch: 'a' })
        ));
    }
}
