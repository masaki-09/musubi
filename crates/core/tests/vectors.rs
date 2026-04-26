//! Integration tests: end-to-end encrypt/decrypt round-trips and golden
//! ciphertext stability.

use musubi_core::{decrypt, encrypt, Alphabet, Ciphertext, Key, MusubiError, Relation};
use rand::SeedableRng;

const ROUND_TRIP_PLAINTEXTS: &[&str] = &[
    "あ",
    "あいしてる",
    "musubi",
    "Hello, musubi!",
    "ABCabc あいう ぱぴぷ ゃゅょ",
    "0123456789",
    "ががが",
    "the quick brown fox jumps over the lazy dog",
];

fn key_for_seed(seed: u64) -> Key {
    let alphabet = Alphabet::default_v1();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    Key::random(&alphabet, &mut rng)
}

#[test]
fn round_trip_every_anchor_position() {
    let key = key_for_seed(0xCAFE_BABE);
    for plaintext in ROUND_TRIP_PLAINTEXTS {
        let n = plaintext.chars().count();
        for anchor in 0..n {
            let cipher = encrypt(plaintext, &key, anchor)
                .unwrap_or_else(|e| panic!("encrypt failed for {plaintext:?}: {e}"));
            let decoded = decrypt(&cipher, &key)
                .unwrap_or_else(|e| panic!("decrypt failed for {plaintext:?}: {e}"));
            assert_eq!(decoded, *plaintext, "anchor={anchor}");
        }
    }
}

#[test]
fn round_trip_through_json() {
    let key = key_for_seed(0xFEED_FACE);
    for plaintext in ROUND_TRIP_PLAINTEXTS {
        let cipher = encrypt(plaintext, &key, 0).unwrap();
        let json = serde_json::to_string(&cipher).unwrap();
        let parsed: Ciphertext = serde_json::from_str(&json).unwrap();
        let decoded = decrypt(&parsed, &key).unwrap();
        assert_eq!(decoded, *plaintext);
    }
}

#[test]
fn key_json_round_trip() {
    let alphabet = Alphabet::default_v1();
    let key = key_for_seed(0xBEAD_C0DE);
    let serialized = key.to_json();
    let parsed = Key::from_json(&serialized, &alphabet).unwrap();
    assert_eq!(parsed.alphabet_id(), key.alphabet_id());
    for rank in 0..key.len() {
        assert_eq!(parsed.char_at(rank), key.char_at(rank));
    }
}

#[test]
fn out_of_alphabet_chars_are_rejected() {
    let key = key_for_seed(1);
    let result = encrypt("こんにちは、世界", &key, 0);
    assert!(matches!(
        result,
        Err(MusubiError::CharOutsideAlphabet { .. })
    ));
}

/// `same` and `mirror` relations are pattern-detected when applicable;
/// otherwise the relation is a `shift`.
#[test]
fn relation_kinds_match_expectation() {
    let key = key_for_seed(2);
    // Repeated characters → Same.
    let cipher = encrypt("aa", &key, 0).unwrap();
    assert!(matches!(cipher.relations[1], Some(Relation::Same { .. })));

    // Distinct characters with non-mirror ranks → Shift (overwhelmingly likely
    // for a random key over a 175-char alphabet).
    let cipher = encrypt("ab", &key, 0).unwrap();
    assert!(matches!(cipher.relations[1], Some(Relation::Shift { .. })));
}

/// Ciphertext anchor slot is always `None`; non-anchor slots are always `Some`.
#[test]
fn anchor_slot_invariant() {
    let key = key_for_seed(3);
    let plaintext = "hello";
    for anchor in 0..plaintext.len() {
        let cipher = encrypt(plaintext, &key, anchor).unwrap();
        for (i, rel) in cipher.relations.iter().enumerate() {
            if i == anchor {
                assert!(rel.is_none(), "anchor slot must be None");
            } else {
                assert!(rel.is_some(), "non-anchor slot must be Some");
            }
        }
    }
}
