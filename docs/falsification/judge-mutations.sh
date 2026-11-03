#!/usr/bin/env bash
# JUDGE-1 mutant elimination (JUDGE-SPEC r1, JUDGE-SUBTYPE r1, JUDGE-SARIF r0,
# JUDGE-RECEIPT r0, JUDGE-SUBTYPE r2, JUDGE-CONTRACTS r0). Each operator (JM-M1..M6, JS-M1..M4,
# SF-M1..M3, RM-1..4, R2-M1..M6, JC-M1..M7) must first COMPILE (else INVALID), then is
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
assert t.count(old) == 1, f"mutation anchor must match exactly once, found {t.count(old)}: {old!r}"
open(p, "w").write(t.replace(old, new, 1))
EOF
}

run_one() { # name old new killer_test [test_target]
  name=$1; old=$2; new=$3; killer=$4; target=${5:-judge_test}
  cp "$J.orig" "$J"
  # An anchor that no longer matches mutates nothing: INVALID, never a verdict.
  if ! mutate "$old" "$new"; then
    echo "$name: INVALID (anchor does not match exactly once)"; fail=$((fail+1)); return
  fi
  # A mutant that does not build proves nothing: it is INVALID, never KILLED.
  if ! cargo build -q 2>/dev/null; then
    echo "$name: INVALID (mutant does not compile)"; fail=$((fail+1)); return
  fi
  if cargo test -q --test "$target" "$killer" 2>&1 | grep -q "test result: ok"; then
    echo "$name: SURVIVED (killer $killer still passes)"; fail=$((fail+1))
  else
    echo "$name: KILLED by $killer"; pass=$((pass+1))
  fi
}

# JM-M1 drop removal detection
run_one JM-M1 'None => out.push(Witness {' 'None => drop(Witness {' jf_f01_removal_is_violated_with_named_fields
# JM-M2 drop retype detection
run_one JM-M2 'if type_str(pv) != type_str(qv) {' 'if false && type_str(pv) != type_str(qv) {' jf_f02_retype_is_violated_with_both_types
# JM-M3 unparsable input treated as HOLDS
run_one JM-M3 '(Err(_), _) | (_, Err(_)) => ("NOT EVALUATED", Some("unparsable"), Vec::new(), Some(pre_text), Stated::default()),' '(Err(_), _) | (_, Err(_)) => ("HOLDS", None, Vec::new(), Some(pre_text), Stated::default()),' jf_f05_unparsable_is_not_evaluated
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
run_one JM-M6 'BaseRead::NoFile => ("NOT EVALUATED", Some("new_contract"), Vec::new(), None, Stated::default()),' 'BaseRead::NoFile => ("HOLDS", None, Vec::new(), None, Stated::default()),' jf_f06_new_contract_is_not_evaluated

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

# JUDGE-SARIF r0 (docs/falsification/SARIF-SPEC.md)
# SF-M1 results dropped from the SARIF log
run_one SF-M1 '"results": results,' '"results": Vec::<Value>::new(),' sf_f02_result_message_is_the_text_sentence judge_sarif_test
# SF-M2 NOT EVALUATED emitted as a successful run
run_one SF-M2 'notifications,
        verdict != "NOT EVALUATED",' 'notifications,
        true,' sf_f05_not_evaluated_is_a_failed_execution judge_sarif_test
# SF-M3 base-file line used where the post-file line belongs
run_one SF-M3 'x.post_line.or(x.anchor_line).unwrap_or(1)' 'x.pre_line.unwrap_or(1)' sf_f03_retype_located_at_post_line judge_sarif_test

# JUDGE-RECEIPT r0 (docs/falsification/JUDGE-RECEIPT-SPEC.md)
# RM-1 re-derived body not compared
run_one RM-1 'if rendered != body_text {' 'if false && rendered != body_text {' jr_f03_forged_verdict_with_recomputed_footer_fails judge_receipt_test
# RM-2 base read from base_ref instead of the recorded commit
run_one RM-2 'Some(base_commit) => match base_bytes(root, base_commit, &contract)? {' 'Some(base_commit) => match base_bytes(root, { let _ = base_commit; base_ref.as_str() }, &contract)? {' jr_f05_base_is_pinned_to_the_commit_not_the_ref judge_receipt_test
# RM-3 post contract hash not checked
run_one RM-3 'if post_b3 != recorded_post {' 'if false && post_b3 != recorded_post {' jr_f06_post_substitution_is_input_drift judge_receipt_test
# RM-4 base contract hash not checked
run_one RM-4 'if base_b3.as_deref() != recorded_base.as_deref() {' 'if false && base_b3.as_deref() != recorded_base.as_deref() {' jr_f07_forged_base_hash_is_input_drift judge_receipt_test

# JUDGE-SUBTYPE r2 (docs/falsification/JUDGE-SUBTYPE-R2-SPEC.md)
# R2-M1 combinator check dropped
run_one R2-M1 'Some(qv) if has_combinator(pv) || has_combinator(qv) => {' 'Some(qv) if false && (has_combinator(pv) || has_combinator(qv)) => {' r2_f01_field_moved_into_anyof_is_not_evaluated judge_subtype_r2_test
# R2-M2 closed-object check ignored: every removal VIOLATED
run_one R2-M2 'let closed = is_closed(post);' 'let closed = { let _ = is_closed(post); true };' r2_f04_open_object_removal_is_a_note judge_subtype_r2_test
# R2-M3 old enum values not checked against the new bound
run_one R2-M3 'Some(vals) => vals.iter().any(|v| violates(kw, v, b)),' 'Some(vals) => { let _ = vals; true },' r2_f07_enum_replaced_by_wider_bound_holds judge_subtype_r2_test
# R2-M4 unjudged_construct reported as HOLDS
run_one R2-M4 '("NOT EVALUATED", Some("unjudged_construct"))' '("HOLDS", None)' r2_f01_field_moved_into_anyof_is_not_evaluated judge_subtype_r2_test

# R2-M5 local reference resolution disabled (Amendment A)
run_one R2-M5 'Some(roots) => deref(&roots[side], v, active),' 'Some(_roots) => { let _ = active; (v.clone(), Vec::new()) },' r2_f12_tightened_defs_target_is_violated judge_subtype_r2_test
# R2-M6 implied-type check dropped (Amendment B)
run_one R2-M6 'if type_str(pv) == "unspecified" && type_made_explicit(pv, qv) {' 'if false && type_str(pv) == "unspecified" && type_made_explicit(pv, qv) {' r2_f15_implied_type_made_explicit_is_a_note judge_subtype_r2_test
# JUDGE-CONTRACTS r0 (docs/falsification/JUDGE-CONTRACTS-SPEC.md)
# JC-M1 unchanged-file filter dropped: every matching contract judged
run_one JC-M1 'let changed: Vec<String> = changed.into_iter().filter(|p| matched.contains(p)).collect();' 'let changed: Vec<String> = { let _ = changed; matched.iter().cloned().collect() };' jc_f01_three_schemas_only_changed_ones_judged judge_contracts_test
# JC-M2 aggregate takes the last verdict, not the worst
run_one JC-M2 'Ok(if violated > 0 {' 'Ok(if let Some(o) = ones.last() { match o.verdict { "VIOLATED" => 1, "HOLDS" => 0, _ => 4 } } else if violated > 0 {' jc_f08_violation_dominates_and_both_are_stated judge_contracts_test
# JC-M3 a glob matching nothing passes
run_one JC-M3 'if matched.is_empty() {' 'if false && matched.is_empty() {' jc_f05_glob_matching_nothing_is_an_error judge_contracts_test
# JC-M4 untracked files ignored
run_one JC-M4 'changed.extend(git_paths(root, &["ls-files", "-z", "--others", "--exclude-standard", "--"], &specs)?);' '' jc_f07_untracked_schema_is_judged judge_contracts_test
# JC-M5 a new contract moves the exit code (Amendment A)
run_one JC-M5 'o.verdict == "NOT EVALUATED" && o.reason != Some("new_contract")' 'o.verdict == "NOT EVALUATED"' jc_f03_added_schema_is_new_contract judge_contracts_test
# JC-M6 a deleted contract passes (Amendment A)
run_one JC-M6 '(None, BaseRead::Found(_)) => ("NOT EVALUATED", Some("contract_deleted"),' '(None, BaseRead::Found(_)) => ("HOLDS", None,' jc_f04_deleted_schema_is_not_evaluated judge_contracts_test
# JC-M7 D6 reverted: the removed sentence guesses at readers again (Amendment A)
run_one JC-M7 'Instances that carry it, valid before, are now rejected; keep the field, or version the change.' 'Consumers reading this field will break; keep the field, or ship this as a new major version and migrate them.' jc_f10_voice_states_structure_not_readers judge_contracts_test
cp "$J.orig" "$J"; rm "$J.orig"
cargo build -q 2>/dev/null
echo "mutants killed: $pass of $((pass+fail))"
[ "$fail" -eq 0 ]
