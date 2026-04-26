//! Relation types — the building blocks of relational ciphertext.
//!
//! Each non-anchor character in the plaintext is described by a [`Relation`]:
//! a *kind* (same / shift / mirror) and a *reference* to another (already
//! decoded) position whose rank the relation transforms.

use serde::{Deserialize, Serialize};

/// The relation between a character and its reference character.
///
/// All variants are tagged in JSON with a `kind` field. Examples:
///
/// ```json
/// { "kind": "same",   "reference": 2 }
/// { "kind": "shift",  "reference": 2, "delta": -3 }
/// { "kind": "mirror", "reference": 2 }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Relation {
    /// `idx_i = idx_ref` — the character is identical to the referenced one.
    Same {
        /// Position of the referenced character.
        reference: usize,
    },
    /// `idx_i = (idx_ref + delta) mod |Σ|`
    Shift {
        /// Position of the referenced character.
        reference: usize,
        /// Signed offset in the alphabet's permuted order.
        delta: i32,
    },
    /// `idx_i = (- idx_ref) mod |Σ|` — mirror around rank 0.
    Mirror {
        /// Position of the referenced character.
        reference: usize,
    },
}

impl Relation {
    /// Position this relation references.
    #[must_use]
    pub fn reference(self) -> usize {
        match self {
            Self::Same { reference }
            | Self::Mirror { reference }
            | Self::Shift { reference, .. } => reference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip_same() {
        let r = Relation::Same { reference: 2 };
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, r#"{"kind":"same","reference":2}"#);
        let back: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn json_round_trip_shift() {
        let r = Relation::Shift {
            reference: 0,
            delta: -7,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, r#"{"kind":"shift","reference":0,"delta":-7}"#);
        let back: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn json_round_trip_mirror() {
        let r = Relation::Mirror { reference: 5 };
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, r#"{"kind":"mirror","reference":5}"#);
        let back: Relation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }
}
