//! `encrypt` / `decrypt` and the [`Ciphertext`] container.

use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{MusubiError, Result};
use crate::key::Key;
use crate::relation::Relation;

/// On-disk format version for [`Ciphertext`].
pub const FORMAT_VERSION: u32 = 1;

/// A complete musubi ciphertext.
///
/// The plaintext can be reconstructed from `(anchor, relations)` together
/// with the [`Key`]. Each non-anchor position has exactly one [`Relation`];
/// the anchor's slot is `None`.
///
/// In v0.2 the optional [`ext`](Self::ext) field carries a backwards-
/// compatible extension that records the mapping from plaintext order to
/// ciphertext positions when noise (迷い糸 / 「ダミー文字」) is injected.
/// Ciphertexts without injected noise omit `ext` entirely; v0.1 decoders
/// then read them as before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ciphertext {
    /// Format version (currently [`FORMAT_VERSION`]).
    pub version: u32,
    /// Identifier of the alphabet this ciphertext was produced for.
    pub alphabet: String,
    /// Total number of slots in [`relations`](Self::relations) — equals
    /// the plaintext length when no noise was injected, or
    /// `plaintext_len + noise` when noise was used.
    pub length: usize,
    /// The revealed character that anchors the chain.
    pub anchor: Anchor,
    /// `relations.len() == length`. The slot at [`Anchor::position`] is `None`.
    pub relations: Vec<Option<Relation>>,
    /// Optional v0.2 extension data. Absent for noise-free ciphertexts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ext: Option<CiphertextExt>,
}

/// Optional v0.2 ciphertext extension.
///
/// When a ciphertext was produced with noise injection (迷い糸), this
/// struct records which ciphertext positions hold the real plaintext
/// (in plaintext order). Absent when no noise was injected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CiphertextExt {
    /// `plaintext_indices[k] = j` means the `k`-th plaintext character is
    /// stored at ciphertext position `j`. `len()` equals the original
    /// plaintext length; positions not listed are noise (dummy) entries.
    pub plaintext_indices: Vec<usize>,
}

/// The plaintext-revealing anchor — position and character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    /// Index of the anchor in the ciphertext (which is also a plaintext
    /// position; the anchor is always a real character, never noise).
    pub position: usize,
    /// Plain (unencrypted) character at that position.
    pub character: char,
}

/// Encrypt `plaintext` with `key`, revealing the character at `anchor_position`.
///
/// Each non-anchor position is described relative to the position one step
/// closer to the anchor. The chosen relation kind for position `i` is:
///
/// - [`Relation::Same`] if `chars[i] == chars[ref]`
/// - [`Relation::Mirror`] if `(rank_i + rank_ref) ≡ 0 (mod |Σ|)`
/// - [`Relation::Shift`] otherwise, with `delta` normalized to `[-|Σ|/2, |Σ|/2]`
///
/// # Errors
///
/// Returns [`MusubiError::EmptyPlaintext`] if `plaintext` is empty,
/// [`MusubiError::AnchorOutOfRange`] if `anchor_position` is past the end,
/// or [`MusubiError::CharOutsideAlphabet`] if `plaintext` contains a
/// character that is not in the key's alphabet.
pub fn encrypt(plaintext: &str, key: &Key, anchor_position: usize) -> Result<Ciphertext> {
    let chars: Vec<char> = plaintext.chars().collect();
    let n = chars.len();
    if n == 0 {
        return Err(MusubiError::EmptyPlaintext);
    }
    if anchor_position >= n {
        return Err(MusubiError::AnchorOutOfRange {
            position: anchor_position,
            length: n,
        });
    }
    for &c in &chars {
        if key.rank_of(c).is_none() {
            return Err(MusubiError::CharOutsideAlphabet { ch: c });
        }
    }

    let mut relations: Vec<Option<Relation>> = vec![None; n];
    for i in (0..anchor_position).rev() {
        let ref_idx = i + 1;
        relations[i] = Some(make_relation(chars[i], chars[ref_idx], ref_idx, key));
    }
    for i in (anchor_position + 1)..n {
        let ref_idx = i - 1;
        relations[i] = Some(make_relation(chars[i], chars[ref_idx], ref_idx, key));
    }

    Ok(Ciphertext {
        version: FORMAT_VERSION,
        alphabet: key.alphabet_id().to_string(),
        length: n,
        anchor: Anchor {
            position: anchor_position,
            character: chars[anchor_position],
        },
        relations,
        ext: None,
    })
}

/// Encrypt `plaintext` with `key`, weaving a random spanning tree rooted at
/// the anchor (the "多重結び / chain" encoder).
///
/// Unlike [`encrypt`], which always references the immediately adjacent
/// position, this encoder visits non-anchor positions in a random order and
/// references *any* already-resolved position. The resulting reference graph
/// is a uniformly random tree rooted at `anchor_position`.
///
/// The output ciphertext has the same on-disk format as [`encrypt`]'s output
/// (`version = 1`); decoders need no changes — [`decrypt`] already accepts
/// any acyclic reference graph rooted at the anchor.
///
/// # Errors
///
/// Same as [`encrypt`].
pub fn encrypt_chain<R: RngCore>(
    plaintext: &str,
    key: &Key,
    anchor_position: usize,
    rng: &mut R,
) -> Result<Ciphertext> {
    let chars: Vec<char> = plaintext.chars().collect();
    let n = chars.len();
    if n == 0 {
        return Err(MusubiError::EmptyPlaintext);
    }
    if anchor_position >= n {
        return Err(MusubiError::AnchorOutOfRange {
            position: anchor_position,
            length: n,
        });
    }
    for &c in &chars {
        if key.rank_of(c).is_none() {
            return Err(MusubiError::CharOutsideAlphabet { ch: c });
        }
    }

    let mut relations: Vec<Option<Relation>> = vec![None; n];

    // Random visit order over non-anchor positions.
    let mut pending: Vec<usize> = (0..n).filter(|&i| i != anchor_position).collect();
    pending.shuffle(rng);

    let mut resolved: Vec<usize> = Vec::with_capacity(n);
    resolved.push(anchor_position);

    for i in pending {
        // Pick a uniformly random already-resolved reference.
        let &ref_idx = resolved
            .choose(rng)
            .expect("resolved always contains at least the anchor");
        relations[i] = Some(make_relation(chars[i], chars[ref_idx], ref_idx, key));
        resolved.push(i);
    }

    Ok(Ciphertext {
        version: FORMAT_VERSION,
        alphabet: key.alphabet_id().to_string(),
        length: n,
        anchor: Anchor {
            position: anchor_position,
            character: chars[anchor_position],
        },
        relations,
        ext: None,
    })
}

/// Encrypt `plaintext` with `key`, weaving a random spanning tree rooted at
/// the anchor and *injecting `noise` dummy characters* (迷い糸 / "wandering
/// threads") into the resulting ciphertext.
///
/// When `noise == 0` this is equivalent to [`encrypt_chain`]. When
/// `noise > 0`:
///
/// - The ciphertext has `length = plaintext.chars().count() + noise`.
/// - `noise` extra positions hold uniformly-random dummy characters from
///   the alphabet. They are interleaved with the plaintext positions
///   uniformly at random.
/// - The reference graph is a uniformly random spanning tree over **all**
///   `length` positions, rooted at the anchor.
/// - The ciphertext carries [`CiphertextExt::plaintext_indices`] so that
///   the receiver (with the right key) can identify and extract the real
///   plaintext positions after decoding the full graph.
///
/// Without the key, an attacker cannot tell which entries are plaintext
/// and which are noise — the true plaintext length is hidden.
///
/// # Errors
///
/// Same as [`encrypt`].
pub fn encrypt_woven<R: RngCore>(
    plaintext: &str,
    key: &Key,
    anchor_position: usize,
    noise: usize,
    rng: &mut R,
) -> Result<Ciphertext> {
    if noise == 0 {
        return encrypt_chain(plaintext, key, anchor_position, rng);
    }

    let chars: Vec<char> = plaintext.chars().collect();
    let n = chars.len();
    if n == 0 {
        return Err(MusubiError::EmptyPlaintext);
    }
    if anchor_position >= n {
        return Err(MusubiError::AnchorOutOfRange {
            position: anchor_position,
            length: n,
        });
    }
    for &c in &chars {
        if key.rank_of(c).is_none() {
            return Err(MusubiError::CharOutsideAlphabet { ch: c });
        }
    }

    let total_len = n + noise;

    // Decide which ciphertext slots hold the real plaintext (in plaintext
    // order). plaintext_indices[k] = j means plaintext[k] is at slot j.
    let mut all_slots: Vec<usize> = (0..total_len).collect();
    all_slots.shuffle(rng);
    let plaintext_indices: Vec<usize> = all_slots[..n].to_vec();
    let noise_slots: Vec<usize> = all_slots[n..].to_vec();

    // Build the full character array: real plaintext chars at their slots,
    // noise slots filled with uniformly random characters from Σ.
    let alpha_len = key.len();
    let mut full_chars: Vec<char> = vec!['\0'; total_len];
    for (k, &j) in plaintext_indices.iter().enumerate() {
        full_chars[j] = chars[k];
    }
    for &j in &noise_slots {
        let rank = rng.gen_range(0..alpha_len);
        full_chars[j] = key
            .char_at(rank)
            .expect("rank drawn from gen_range(0..alpha_len) is always valid");
    }

    // Anchor sits at the slot that holds the original anchor character.
    let cipher_anchor = plaintext_indices[anchor_position];
    let anchor_char = full_chars[cipher_anchor];

    // Build a uniformly random spanning tree over all `total_len` slots,
    // rooted at `cipher_anchor`.
    let mut relations: Vec<Option<Relation>> = vec![None; total_len];
    let mut pending: Vec<usize> = (0..total_len).filter(|&i| i != cipher_anchor).collect();
    pending.shuffle(rng);

    let mut resolved: Vec<usize> = Vec::with_capacity(total_len);
    resolved.push(cipher_anchor);

    for i in pending {
        let &ref_idx = resolved
            .choose(rng)
            .expect("resolved always contains at least the anchor");
        relations[i] = Some(make_relation(
            full_chars[i],
            full_chars[ref_idx],
            ref_idx,
            key,
        ));
        resolved.push(i);
    }

    Ok(Ciphertext {
        version: FORMAT_VERSION,
        alphabet: key.alphabet_id().to_string(),
        length: total_len,
        anchor: Anchor {
            position: cipher_anchor,
            character: anchor_char,
        },
        relations,
        ext: Some(CiphertextExt { plaintext_indices }),
    })
}

fn make_relation(c: char, ref_c: char, ref_idx: usize, key: &Key) -> Relation {
    let n = key.len() as i64;
    let r0 = key.rank_of(ref_c).expect("ref_c was validated") as i64;
    let r1 = key.rank_of(c).expect("c was validated") as i64;
    if c == ref_c {
        Relation::Same { reference: ref_idx }
    } else if (r0 + r1).rem_euclid(n) == 0 {
        Relation::Mirror { reference: ref_idx }
    } else {
        let raw = (r1 - r0).rem_euclid(n);
        let delta = if raw > n / 2 { raw - n } else { raw };
        Relation::Shift {
            reference: ref_idx,
            delta: delta as i32,
        }
    }
}

/// Decrypt `cipher` with `key`, recovering the original plaintext.
///
/// # Errors
///
/// Returns [`MusubiError::MalformedCiphertext`] for any structural problem
/// (bad version, wrong relations length, missing relation, dangling
/// reference, …), [`MusubiError::AlphabetMismatch`] if the ciphertext
/// belongs to a different alphabet than the key, or
/// [`MusubiError::CharOutsideAlphabet`] if the anchor or a recovered
/// character is not in the key's alphabet.
pub fn decrypt(cipher: &Ciphertext, key: &Key) -> Result<String> {
    if cipher.version != FORMAT_VERSION {
        return Err(MusubiError::MalformedCiphertext {
            reason: format!("unsupported version {}", cipher.version),
        });
    }
    if cipher.alphabet != key.alphabet_id() {
        return Err(MusubiError::AlphabetMismatch {
            expected: key.alphabet_id().to_string(),
            got: cipher.alphabet.clone(),
        });
    }
    let n = cipher.length;
    if cipher.relations.len() != n {
        return Err(MusubiError::MalformedCiphertext {
            reason: format!(
                "relations length {} does not match declared length {}",
                cipher.relations.len(),
                n
            ),
        });
    }
    if n == 0 {
        return Err(MusubiError::MalformedCiphertext {
            reason: "ciphertext has zero length".to_string(),
        });
    }
    let anchor_pos = cipher.anchor.position;
    if anchor_pos >= n {
        return Err(MusubiError::AnchorOutOfRange {
            position: anchor_pos,
            length: n,
        });
    }
    if cipher.relations[anchor_pos].is_some() {
        return Err(MusubiError::MalformedCiphertext {
            reason: format!("anchor position {anchor_pos} must have null relation"),
        });
    }
    if key.rank_of(cipher.anchor.character).is_none() {
        return Err(MusubiError::CharOutsideAlphabet {
            ch: cipher.anchor.character,
        });
    }

    let mut chars: Vec<Option<char>> = vec![None; n];
    chars[anchor_pos] = Some(cipher.anchor.character);
    let mut remaining = n - 1;

    while remaining > 0 {
        let mut progress = false;
        for i in 0..n {
            if chars[i].is_some() {
                continue;
            }
            let rel =
                cipher.relations[i]
                    .as_ref()
                    .ok_or_else(|| MusubiError::MalformedCiphertext {
                        reason: format!("missing relation at position {i}"),
                    })?;
            let ref_idx = rel.reference();
            if ref_idx >= n {
                return Err(MusubiError::MalformedCiphertext {
                    reason: format!(
                        "relation at position {i} references out-of-range index {ref_idx}"
                    ),
                });
            }
            let Some(ref_char) = chars[ref_idx] else {
                continue;
            };
            chars[i] = Some(apply_relation(*rel, ref_char, key)?);
            progress = true;
            remaining -= 1;
        }
        if !progress {
            return Err(MusubiError::MalformedCiphertext {
                reason: "relation graph has cycles or unreachable positions".to_string(),
            });
        }
    }

    if let Some(ext) = &cipher.ext {
        validate_ext_indices(&ext.plaintext_indices, n, anchor_pos)?;
        let plaintext: String = ext
            .plaintext_indices
            .iter()
            .map(|&i| chars[i].expect("filled in loop"))
            .collect();
        return Ok(plaintext);
    }

    let plaintext: String = chars
        .into_iter()
        .map(|c| c.expect("filled in loop"))
        .collect();
    Ok(plaintext)
}

fn validate_ext_indices(indices: &[usize], length: usize, anchor_pos: usize) -> Result<()> {
    if indices.is_empty() {
        return Err(MusubiError::MalformedCiphertext {
            reason: "ext.plaintext_indices must not be empty".to_string(),
        });
    }
    if indices.len() > length {
        return Err(MusubiError::MalformedCiphertext {
            reason: format!(
                "ext.plaintext_indices length {} exceeds ciphertext length {length}",
                indices.len()
            ),
        });
    }
    let mut seen = vec![false; length];
    for &idx in indices {
        if idx >= length {
            return Err(MusubiError::MalformedCiphertext {
                reason: format!(
                    "ext.plaintext_indices contains out-of-range index {idx} (length {length})"
                ),
            });
        }
        if seen[idx] {
            return Err(MusubiError::MalformedCiphertext {
                reason: format!("ext.plaintext_indices contains duplicate index {idx}"),
            });
        }
        seen[idx] = true;
    }
    if !indices.contains(&anchor_pos) {
        return Err(MusubiError::MalformedCiphertext {
            reason: format!(
                "ext.plaintext_indices does not contain the anchor position {anchor_pos}"
            ),
        });
    }
    Ok(())
}

fn apply_relation(rel: Relation, ref_char: char, key: &Key) -> Result<char> {
    let n = key.len() as i64;
    let r_ref = key
        .rank_of(ref_char)
        .ok_or(MusubiError::CharOutsideAlphabet { ch: ref_char })? as i64;
    let target_rank = match rel {
        Relation::Same { .. } => r_ref,
        Relation::Shift { delta, .. } => (r_ref + i64::from(delta)).rem_euclid(n),
        Relation::Mirror { .. } => (-r_ref).rem_euclid(n),
    };
    key.char_at(target_rank as usize)
        .ok_or_else(|| MusubiError::MalformedCiphertext {
            reason: format!("computed rank {target_rank} out of range"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::Alphabet;
    use rand::SeedableRng;

    fn fresh_key(seed: u64) -> (Alphabet, Key) {
        let alphabet = Alphabet::default_v1();
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let key = Key::random(&alphabet, &mut rng);
        (alphabet, key)
    }

    #[test]
    fn encrypt_decrypt_round_trip_simple() {
        let (_a, key) = fresh_key(0xA110_CA7E);
        let plaintext = "musubi";
        let cipher = encrypt(plaintext, &key, 0).unwrap();
        assert_eq!(cipher.length, plaintext.chars().count());
        assert_eq!(cipher.relations[0], None);
        let decrypted = decrypt(&cipher, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn round_trip_for_every_anchor_position() {
        let (_a, key) = fresh_key(0xDEAD_BEEF);
        let plaintext = "あいしてる";
        let n = plaintext.chars().count();
        for anchor in 0..n {
            let cipher = encrypt(plaintext, &key, anchor).unwrap();
            assert_eq!(decrypted_from(&cipher, &key), plaintext, "anchor={anchor}");
        }
    }

    fn decrypted_from(cipher: &Ciphertext, key: &Key) -> String {
        decrypt(cipher, key).unwrap()
    }

    #[test]
    fn repeated_chars_use_same_relation() {
        let (_a, key) = fresh_key(1);
        let plaintext = "ががが";
        let cipher = encrypt(plaintext, &key, 1).unwrap();
        assert!(matches!(cipher.relations[0], Some(Relation::Same { .. })));
        assert!(matches!(cipher.relations[2], Some(Relation::Same { .. })));
        assert_eq!(decrypt(&cipher, &key).unwrap(), plaintext);
    }

    #[test]
    fn empty_plaintext_errors() {
        let (_a, key) = fresh_key(2);
        assert!(matches!(
            encrypt("", &key, 0),
            Err(MusubiError::EmptyPlaintext)
        ));
    }

    #[test]
    fn anchor_out_of_range_errors() {
        let (_a, key) = fresh_key(3);
        assert!(matches!(
            encrypt("abc", &key, 5),
            Err(MusubiError::AnchorOutOfRange { .. })
        ));
    }

    #[test]
    fn out_of_alphabet_char_errors() {
        let (_a, key) = fresh_key(4);
        // 「、」 and 「世」 are not in default-v1
        assert!(matches!(
            encrypt("こんにちは、世界", &key, 0),
            Err(MusubiError::CharOutsideAlphabet { .. })
        ));
    }

    #[test]
    fn version_mismatch_rejects_decrypt() {
        let (_a, key) = fresh_key(5);
        let mut cipher = encrypt("hi", &key, 0).unwrap();
        cipher.version = 999;
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::MalformedCiphertext { .. })
        ));
    }

    #[test]
    fn alphabet_mismatch_rejects_decrypt() {
        let (_a, key) = fresh_key(6);
        let mut cipher = encrypt("hi", &key, 0).unwrap();
        cipher.alphabet = "other".to_string();
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::AlphabetMismatch { .. })
        ));
    }

    #[test]
    fn chain_encoder_round_trips_for_every_anchor_position() {
        let (_a, key) = fresh_key(0x00C0_FFEE);
        let plaintext = "あいしてる";
        let n = plaintext.chars().count();
        for anchor in 0..n {
            let mut rng = rand::rngs::StdRng::seed_from_u64(0xBEEF + anchor as u64);
            let cipher = encrypt_chain(plaintext, &key, anchor, &mut rng).unwrap();
            assert_eq!(cipher.length, n);
            assert_eq!(cipher.anchor.position, anchor);
            assert_eq!(cipher.relations[anchor], None);
            for (i, r) in cipher.relations.iter().enumerate() {
                if i == anchor {
                    continue;
                }
                assert!(r.is_some(), "missing relation at {i}");
            }
            assert_eq!(decrypt(&cipher, &key).unwrap(), plaintext);
        }
    }

    #[test]
    fn chain_encoder_is_deterministic_with_seeded_rng() {
        let (_a, key) = fresh_key(0xD00D);
        let plaintext = "Hello, musubi!";
        let mut rng_a = rand::rngs::StdRng::seed_from_u64(7);
        let mut rng_b = rand::rngs::StdRng::seed_from_u64(7);
        let a = encrypt_chain(plaintext, &key, 3, &mut rng_a).unwrap();
        let b = encrypt_chain(plaintext, &key, 3, &mut rng_b).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn chain_encoder_produces_acyclic_graph_rooted_at_anchor() {
        let (_a, key) = fresh_key(0xACED);
        let plaintext = "あいうえおかきくけこ";
        let mut rng = rand::rngs::StdRng::seed_from_u64(123);
        let cipher = encrypt_chain(plaintext, &key, 4, &mut rng).unwrap();
        // BFS from the anchor along reverse-reference edges should reach every node.
        let n = cipher.length;
        let mut reachable = vec![false; n];
        reachable[cipher.anchor.position] = true;
        let mut progress = true;
        while progress {
            progress = false;
            for (i, r) in cipher.relations.iter().enumerate() {
                if reachable[i] {
                    continue;
                }
                if let Some(rel) = r {
                    if reachable[rel.reference()] {
                        reachable[i] = true;
                        progress = true;
                    }
                }
            }
        }
        assert!(reachable.iter().all(|&b| b), "all nodes reachable");
    }

    #[test]
    fn chain_encoder_rejects_empty_and_out_of_range() {
        let (_a, key) = fresh_key(11);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        assert!(matches!(
            encrypt_chain("", &key, 0, &mut rng),
            Err(MusubiError::EmptyPlaintext)
        ));
        assert!(matches!(
            encrypt_chain("abc", &key, 9, &mut rng),
            Err(MusubiError::AnchorOutOfRange { .. })
        ));
    }

    #[test]
    fn ciphertext_round_trips_through_json() {
        let (_a, key) = fresh_key(7);
        let cipher = encrypt("Hello, musubi!", &key, 7).unwrap();
        let json = serde_json::to_string_pretty(&cipher).unwrap();
        let parsed: Ciphertext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cipher);
    }

    #[test]
    fn legacy_ciphertext_without_ext_field_still_decodes() {
        // A v0.1-style JSON document — note: no `ext` field present.
        let (_a, key) = fresh_key(0x0BAD_F00D);
        let cipher = encrypt("musubi", &key, 0).unwrap();
        let mut json: serde_json::Value = serde_json::to_value(&cipher).unwrap();
        // serializer omits None ext, but ensure removal even if a tool added it.
        json.as_object_mut().unwrap().remove("ext");
        let parsed: Ciphertext = serde_json::from_value(json).unwrap();
        assert!(parsed.ext.is_none());
        assert_eq!(decrypt(&parsed, &key).unwrap(), "musubi");
    }

    #[test]
    fn ciphertext_omits_ext_field_when_absent() {
        let (_a, key) = fresh_key(0xABCD);
        let cipher = encrypt("hi", &key, 0).unwrap();
        let json = serde_json::to_string(&cipher).unwrap();
        assert!(!json.contains("\"ext\""), "ext should be omitted: {json}");
    }

    #[test]
    fn woven_with_zero_noise_is_chain() {
        let (_a, key) = fresh_key(0xFACE);
        let plaintext = "あいしてる";
        let mut rng_a = rand::rngs::StdRng::seed_from_u64(99);
        let mut rng_b = rand::rngs::StdRng::seed_from_u64(99);
        let a = encrypt_woven(plaintext, &key, 2, 0, &mut rng_a).unwrap();
        let b = encrypt_chain(plaintext, &key, 2, &mut rng_b).unwrap();
        assert_eq!(a, b);
        assert!(a.ext.is_none());
    }

    #[test]
    fn woven_round_trips_for_every_anchor_position() {
        let (_a, key) = fresh_key(0x1234_5678);
        let plaintext = "あいしてる";
        let n = plaintext.chars().count();
        for anchor in 0..n {
            let mut rng = rand::rngs::StdRng::seed_from_u64(0xCAFE + anchor as u64);
            let cipher = encrypt_woven(plaintext, &key, anchor, 4, &mut rng).unwrap();
            assert_eq!(cipher.length, n + 4);
            assert!(cipher.ext.is_some());
            let ext = cipher.ext.as_ref().unwrap();
            assert_eq!(ext.plaintext_indices.len(), n);
            assert!(
                ext.plaintext_indices.contains(&cipher.anchor.position),
                "anchor must be in plaintext_indices"
            );
            assert_eq!(decrypt(&cipher, &key).unwrap(), plaintext);
        }
    }

    #[test]
    fn woven_is_deterministic_with_seeded_rng() {
        let (_a, key) = fresh_key(0xD00D);
        let plaintext = "Hello, musubi!";
        let mut rng_a = rand::rngs::StdRng::seed_from_u64(7);
        let mut rng_b = rand::rngs::StdRng::seed_from_u64(7);
        let a = encrypt_woven(plaintext, &key, 3, 5, &mut rng_a).unwrap();
        let b = encrypt_woven(plaintext, &key, 3, 5, &mut rng_b).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn woven_hides_plaintext_length() {
        let (_a, key) = fresh_key(0xBABE);
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let cipher = encrypt_woven("あいしてる", &key, 2, 7, &mut rng).unwrap();
        // Plaintext length (5) is not directly exposed; ciphertext length is 12.
        assert_eq!(cipher.length, 12);
    }

    #[test]
    fn woven_round_trips_through_json() {
        let (_a, key) = fresh_key(0xDEED);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let cipher = encrypt_woven("musubi", &key, 0, 3, &mut rng).unwrap();
        let json = serde_json::to_string_pretty(&cipher).unwrap();
        assert!(json.contains("\"plaintext_indices\""));
        let parsed: Ciphertext = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, cipher);
        assert_eq!(decrypt(&parsed, &key).unwrap(), "musubi");
    }

    #[test]
    fn woven_rejects_invalid_ext() {
        let (_a, key) = fresh_key(0xBEAD);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let mut cipher = encrypt_woven("musubi", &key, 0, 2, &mut rng).unwrap();

        // Out-of-range index.
        let original = cipher.ext.clone().unwrap();
        cipher.ext = Some(CiphertextExt {
            plaintext_indices: vec![0, 1, 2, 999],
        });
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::MalformedCiphertext { .. })
        ));

        // Duplicate index.
        cipher.ext = Some(CiphertextExt {
            plaintext_indices: vec![0, 0, 1],
        });
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::MalformedCiphertext { .. })
        ));

        // Empty.
        cipher.ext = Some(CiphertextExt {
            plaintext_indices: vec![],
        });
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::MalformedCiphertext { .. })
        ));

        // Anchor not in indices.
        let anchor_pos = cipher.anchor.position;
        let other_indices: Vec<usize> = original
            .plaintext_indices
            .iter()
            .copied()
            .filter(|&i| i != anchor_pos)
            .collect();
        cipher.ext = Some(CiphertextExt {
            plaintext_indices: other_indices,
        });
        assert!(matches!(
            decrypt(&cipher, &key),
            Err(MusubiError::MalformedCiphertext { .. })
        ));
    }

    #[test]
    fn woven_rejects_empty_plaintext() {
        let (_a, key) = fresh_key(11);
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        assert!(matches!(
            encrypt_woven("", &key, 0, 3, &mut rng),
            Err(MusubiError::EmptyPlaintext)
        ));
    }
}
