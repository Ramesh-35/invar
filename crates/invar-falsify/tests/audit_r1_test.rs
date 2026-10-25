//! AUDIT r1 repair fixture (docs/falsification/AUDIT-SPEC.md r1), added under
//! the founder's ruling of 2026-10-25 after the r0 mutation run: AU-M5
//! (string-literal contents not tokenized) survived as a non-equivalent path.
//! Its r0 killer, au_f07, uses `data["legacy"]`, whose key is also an
//! identifier the identifier scan finds inside the quotes. A key that is not
//! an identifier (`template-id`) is found ONLY by tokenizing the literal.
//! tests/audit_test.rs (r0) is unchanged byte-for-byte.

use invar_falsify::check::{audit_witness, Witness};

#[test]
fn au_f07b_a_non_identifier_key_inside_a_literal_is_a_token() {
    let w = Witness {
        residue: "template-id".to_string(),
        sites: vec!["c.py:1".to_string()],
        missing_from: Some("head.go".to_string()),
        type_change: None,
        read_as: Vec::new(),
    };
    let r = audit_witness(&w, "c.py", "t = row[\"template-id\"]\n", &["head.go".to_string()]);
    assert_eq!(r, Ok(()), "the literal's content is one whole token");
}
