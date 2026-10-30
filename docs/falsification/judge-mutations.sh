#!/usr/bin/env bash
# JUDGE-1 mutant elimination (JUDGE-SPEC r1). Each operator JM-M1..M6 is
# applied to src/judge.rs as a real source mutation; the named killer
# fixture must FAIL under the mutant (mutant KILLED) and the file is
# restored. Any mutant that survives is reported SURVIVED and the run
# exits 1. Run from the repo root.
set -uo pipefail
J=crates/invar-falsify/src/judge.rs
cp "$J" "$J.orig"
pass=0; fail=0

mutate() { python3 - "$J" "$1" "$2" <<'EOF'
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
t = open(p).read()
assert t.count(old) >= 1, f"mutation anchor not found: {old!r}"
open(p, "w").write(t.replace(old, new, 1))
EOF
}

run_one() { # name old new killer_test [test_target]
  name=$1; old=$2; new=$3; killer=$4; target=${5:-judge_test}
  cp "$J.orig" "$J"
  mutate "$old" "$new"
  if cargo test -q --test "$target" "$killer" 2>&1 | grep -q "test result: ok"; then
    echo "$name: SURVIVED (killer $killer still passes)"; fail=$((fail+1))
  else
    echo "$name: KILLED by $killer"; pass=$((pass+1))
  fi
}

# JM-M1 drop removal detection
run_one JM-M1 'None => out.push(Witness {' 'None if false => out.push(Witness {' jf_f01_removal_is_violated_with_named_fields
# JM-M2 drop retype detection
run_one JM-M2 'if type_str(pv) != type_str(qv) {' 'if false && type_str(pv) != type_str(qv) {' jf_f02_retype_is_violated_with_both_types
# JM-M3 unparsable input treated as HOLDS
run_one JM-M3 '(Err(_), _) | (_, Err(_)) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Some(pre_text)),' '(Err(_), _) | (_, Err(_)) => ("HOLDS", None, Vec::new(), Some(pre_text)),' jf_f05_unparsable_is_not_evaluated
# JM-M4 pre read from working tree instead of base ref
run_one JM-M4 'match untar_single(&out.stdout, contract) {
        Some(bytes) => Ok(BaseRead::Found(bytes)),
        None => Ok(BaseRead::NoFile),
    }' 'let _ = &out; match std::fs::read(root.join(contract)) {
        Ok(bytes) => Ok(BaseRead::Found(bytes)),
        Err(_) => Ok(BaseRead::NoFile),
    }' jf_f01_removal_is_violated_with_named_fields
# JM-M5 recursion removed, top level only
run_one JM-M5 'walk(&dotted, pv, qv, out);' 'let _ = &dotted;' jf_f07_nested_removal_has_dotted_path
# JM-M6 missing base contract treated as HOLDS
run_one JM-M6 'BaseRead::NoFile => ("NOT EVALUATED", Some("new_contract"), Vec::new(), None),' 'BaseRead::NoFile => ("HOLDS", None, Vec::new(), None),' jf_f06_new_contract_is_not_evaluated

# JUDGE-SUBTYPE r1 (docs/falsification/JUDGE-SUBTYPE-SPEC.md)
# JS-M1 enum narrowing check dropped
run_one JS-M1 'if narrowed_enum(pv, qv) {' 'if false && narrowed_enum(pv, qv) {' js_f01_enum_value_removed_is_narrowed judge_subtype_test
# JS-M2 bounds comparison inverted (killed from both directions)
run_one JS-M2a 'Some(a) => (is_lower && b > a) || (!is_lower && b < a),' 'Some(a) => (is_lower && b < a) || (!is_lower && b > a),' js_f03_minimum_raised_is_tightened judge_subtype_test
run_one JS-M2b 'Some(a) => (is_lower && b > a) || (!is_lower && b < a),' 'Some(a) => (is_lower && b < a) || (!is_lower && b > a),' js_f04_maximum_raised_holds judge_subtype_test
# JS-M3 required diff dropped
run_one JS-M3 'for name in newly_required(pre, post) {' 'for name in newly_required(pre, post).into_iter().take(0) {' js_f05_optional_promoted_to_required_is_escalated judge_subtype_test
# JS-M4 pattern or format change treated as HOLDS
run_one JS-M4 'if pv.get(kw) != qv.get(kw) && qv.get(kw).is_some() {' 'if false && pv.get(kw) != qv.get(kw) && qv.get(kw).is_some() {' js_f06_pattern_change_is_flagged_not_proven judge_subtype_test

cp "$J.orig" "$J"; rm "$J.orig"
cargo build -q 2>/dev/null
echo "mutants killed: $pass of $((pass+fail))"
[ "$fail" -eq 0 ]
