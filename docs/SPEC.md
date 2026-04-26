# musubi Cipher Specification v1

This document is the **normative** specification for the musubi
relational cipher, version 1. The implementation in
[`musubi-core`](../crates/core) MUST match this specification.

A more intuitive walkthrough lives in [THEORY.md](THEORY.md).

---

## 1. Notation

| Symbol | Meaning |
|---|---|
| `Σ`           | finite, non-empty set of characters (the *alphabet*) |
| `\|Σ\|`       | size of the alphabet |
| `π`           | a permutation `{0, …, \|Σ\|−1} → Σ` (the *key*) |
| `π⁻¹(c)`      | the rank of character `c` under `π`, i.e. its position |
| `m = m₀ m₁ … mₙ₋₁` | plaintext, a sequence of characters in `Σ` |
| `n`           | length of the plaintext |
| `a`           | anchor position, `0 ≤ a < n` |
| `r_i`         | reference position for index `i ≠ a` |
| `idx_i`       | the rank `π⁻¹(m_i)` |

All arithmetic is modulo `\|Σ\|` unless stated otherwise.

## 2. Alphabet Σ

An alphabet is a finite, ordered, duplicate-free sequence of Unicode
scalar values. The order is *canonical*: it is fixed when the alphabet is
constructed and is part of the alphabet's identity.

Every alphabet has a stable string identifier (e.g. `"default-v1"`). Two
alphabets with the same identifier MUST have the same character set in
the same canonical order.

### 2.1 default-v1

The reference alphabet for v1 is identified by `"default-v1"` and has
`\|Σ\| = 175` characters in the order:

1. **Hiragana 清音** (46): `あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん`
2. **Hiragana 濁音** (20): `がぎぐげござじずぜぞだぢづでどばびぶべぼ`
3. **Hiragana 半濁音** (5): `ぱぴぷぺぽ`
4. **Hiragana 小書き仮名** (9): `ぁぃぅぇぉっゃゅょ`
5. **ASCII printable** (95): code points `U+0020`..`U+007E` in numeric order

Any character outside this set is **out-of-alphabet** and MUST be
rejected by the encoder.

## 3. Key (permutation π)

A *key* is a permutation of the canonical character list of an alphabet.
It binds to a single alphabet via the alphabet's identifier; a key for
alphabet *X* MUST NOT be used to decrypt a ciphertext produced for
alphabet *Y*.

### 3.1 Required operations

A key implementation MUST support, in time independent of plaintext length:

- `rank_of(c) → idx`  — given `c ∈ Σ`, return `π⁻¹(c)`.
- `char_at(idx) → c`  — given a valid rank, return the character.
- `len() → \|Σ\|`     — return the alphabet size.

### 3.2 Key serialization

A serialized key is a JSON object with exactly two fields:

```json
{
  "alphabet": "default-v1",
  "permutation": ["…", "…", "…"]
}
```

- `alphabet`: the alphabet identifier the key is for.
- `permutation`: the alphabet's characters in the key's order. The list
  MUST contain every character of the alphabet exactly once.

## 4. Relation

A *relation* describes how to derive one character's rank from another's.

```
shift(k):  idx_i = (idx_{r_i} + k)  mod |Σ|
same:      idx_i =  idx_{r_i}
mirror:    idx_i = (-idx_{r_i})     mod |Σ|
```

A relation also carries the *reference position* `r_i`. JSON encoding:

```json
{ "kind": "shift",  "reference": 0, "delta": -3 }
{ "kind": "same",   "reference": 2 }
{ "kind": "mirror", "reference": 5 }
```

The `kind` field is the discriminator. `shift` SHALL also include
`delta`, an integer in the range `[−⌊\|Σ\|/2⌋, ⌊\|Σ\|/2⌋]`.

## 5. Ciphertext

A ciphertext is a JSON object:

```json
{
  "version":  1,
  "alphabet": "default-v1",
  "length":   5,
  "anchor":   { "position": 2, "character": "し" },
  "relations": [
    { "kind": "shift",  "reference": 1, "delta": -2 },
    { "kind": "shift",  "reference": 2, "delta":  3 },
    null,
    { "kind": "shift",  "reference": 2, "delta":  7 },
    { "kind": "shift",  "reference": 3, "delta": -1 }
  ]
}
```

### 5.1 Field semantics

| Field | Constraint |
|---|---|
| `version`   | MUST equal `1` for this specification. |
| `alphabet`  | The alphabet identifier the ciphertext is for. |
| `length`    | The number of characters in the plaintext, `n ≥ 1`. |
| `anchor.position`  | `0 ≤ position < length`. |
| `anchor.character` | A character in the alphabet (revealed plaintext). |
| `relations` | An array of length exactly `length`. The slot at `anchor.position` MUST be `null`; every other slot MUST be a non-null relation object. |

### 5.2 Reference invariants

For every non-null relation at index `i`:

- `reference < length`
- `reference ≠ i`
- The reference graph MUST be acyclic when traversed from the anchor.
  Every position MUST be reachable from the anchor via reference edges.

A conforming decoder MAY accept any acyclic reference graph; a conforming
encoder MUST emit relations in the canonical form described in §6.

## 6. Encryption (canonical encoder)

Given plaintext `m₀ m₁ … mₙ₋₁` and anchor position `a`:

1. Verify `n ≥ 1` and `a < n`. Otherwise return an error.
2. For each `m_i`, verify `m_i ∈ Σ`. Otherwise return `CharOutsideAlphabet`.
3. Initialize `relations` as an array of `n` nulls.
4. For `i ∈ {a − 1, a − 2, …, 0}` (in this order), set
   `relations[i] = chooseRelation(m_i, m_{i+1}, i+1)`.
5. For `i ∈ {a + 1, a + 2, …, n − 1}` (in this order), set
   `relations[i] = chooseRelation(m_i, m_{i−1}, i−1)`.
6. Emit the ciphertext with `(version=1, alphabet=π.id, length=n, anchor=(a, m_a), relations)`.

`chooseRelation(c, ref_c, ref_idx)` is:

- If `c == ref_c`: `Same { reference: ref_idx }`.
- Else if `(π⁻¹(c) + π⁻¹(ref_c)) ≡ 0 (mod |Σ|)`: `Mirror { reference: ref_idx }`.
- Else: `Shift { reference: ref_idx, delta }` where
  `delta = ((π⁻¹(c) − π⁻¹(ref_c)) mod |Σ|)`,
  normalized to `[−⌊|Σ|/2⌋, ⌊|Σ|/2⌋]`.

## 7. Decryption

Given a ciphertext `C` and a key `π`:

1. Reject if `C.version ≠ 1`.
2. Reject if `C.alphabet ≠ π.id`.
3. Reject if `C.relations.len() ≠ C.length` or `C.length = 0`.
4. Reject if `relations[C.anchor.position] ≠ null` or any other slot is `null`.
5. Reject if `C.anchor.character ∉ Σ`.
6. Initialize `chars[i] = None` for all `i`. Set `chars[C.anchor.position] = Some(C.anchor.character)`.
7. Repeat until every `chars[i]` is `Some`:
   - For each unfilled `i`: if `chars[r_i]` is filled, compute
     `chars[i] = applyRelation(relations[i], chars[r_i])`.
   - If a full pass makes no progress, reject as a malformed reference graph.
8. Return `chars[0] chars[1] … chars[n−1]`.

`applyRelation(rel, ref_char)` computes the target rank from §4 and looks
up the corresponding character via `π`.

## 8. Errors

| Class | When raised |
|---|---|
| `CharOutsideAlphabet`  | A plaintext or ciphertext character is not in `Σ`. |
| `EmptyAlphabet`        | An alphabet was constructed empty. |
| `DuplicateAlphabetChar`| An alphabet was constructed with a duplicate. |
| `PermutationLengthMismatch` | A key permutation has the wrong size. |
| `PermutationNotBijection`   | A key permutation is not bijective. |
| `EmptyPlaintext`       | `encrypt` was called with `n = 0`. |
| `AnchorOutOfRange`     | `a ≥ n`. |
| `AlphabetMismatch`     | Ciphertext and key alphabet identifiers differ. |
| `MalformedCiphertext`  | Any ciphertext structural violation. |

## 9. Versioning

This specification is **version 1** (`version: 1` in ciphertexts). Future
revisions adding new relation kinds, alternative encoders, or extended
alphabet families will increment this version. Implementations MUST
reject ciphertexts with a `version` they do not recognize.

## 10. Security model

musubi is a **toy classical cipher**. It is intentionally vulnerable to
classical cryptanalysis (frequency analysis, repeated-character
detection, etc.). Implementations and users MUST NOT rely on musubi for
confidentiality of any sensitive information.

The threat model is "a curious friend with pen and paper", not a
nation-state attacker. The romance is in the cipher's *theory*, not its
*strength*.
