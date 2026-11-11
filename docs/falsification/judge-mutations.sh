#!/usr/bin/env bash
# JUDGE-1 mutant elimination (JUDGE-SPEC r1, JUDGE-SUBTYPE r1, JUDGE-SARIF r0,
# JUDGE-RECEIPT r0, JUDGE-SUBTYPE r2, JUDGE-CONTRACTS r0, JUDGE-WIDEN r0, COUNTEREXAMPLE r0, JUDGE-R3 r0, COUNTEREXAMPLE r1, COUNTEREXAMPLE r2, JUDGE-D9 r0, JUDGE-FILEPAIR r0, JUDGE-REACH r0, JUDGE-DRAFT r0, JUDGE-ITEMS r0, JUDGE-TESTS r0 and r1, JUDGE-CANON r0 and r1). Each operator (JM-M1..M6, JS-M1..M4,
# SF-M1..M3, RM-1..4, R2-M1..M6, JC-M1..M7, W-M1..M4, CX-M1..M5, R3-M1..M4, K-M1..M8, C-M1..M5, D9-M1..M3, FP-M1..M3, RE-M1..M8, DR-M1..M4, IT-M1..M4, TS-M1..M7, CN-M1..M4, CR1-M1..M5) must first COMPILE (else INVALID), then is
# applied to src/judge.rs (or src/counterexample.rs) as a real source mutation; the named killer
# fixture must FAIL under the mutant (mutant KILLED) and the file is
# restored. Any mutant that survives is reported SURVIVED and the run
# exits 1. Run from the repo root.
set -uo pipefail
J=crates/invar-falsify/src/judge.rs
X=crates/invar-falsify/src/counterexample.rs
P=action/tests.py
cp "$J" "$J.orig"; cp "$X" "$X.orig"; cp "$P" "$P.orig"
pass=0; fail=0

mutate() { python3 - "${3:-$J}" "$1" "$2" <<'EOF'
import sys
p, old, new = sys.argv[1], sys.argv[2], sys.argv[3]
t = open(p).read()
assert t.count(old) == 1, f"mutation anchor must match exactly once, found {t.count(old)}: {old!r}"
open(p, "w").write(t.replace(old, new, 1))
EOF
}

run_one() { # name old new killer_test [test_target] [file]
  name=$1; old=$2; new=$3; killer=$4; target=${5:-judge_test}; file=${6:-$J}
  cp "$J.orig" "$J"; cp "$X.orig" "$X"; cp "$P.orig" "$P"
  # An anchor that no longer matches mutates nothing: INVALID, never a verdict.
  if ! mutate "$old" "$new" "$file"; then
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
run_one JS-M1 '_ if narrowed_enum(pv, qv) => {' '_ if false && narrowed_enum(pv, qv) => {' js_f01_enum_value_removed_is_narrowed judge_subtype_test
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
# JUDGE-WIDEN r0 (docs/falsification/JUDGE-WIDEN-SPEC.md)
# W-M1 widening check dropped: every retype VIOLATED
run_one W-M1 '} else if type_str(pv) != type_str(qv) && widens(pv, qv) {' '} else if false && widens(pv, qv) {' w_f07_b1_002_holds_with_note judge_widen_test
# W-M2 integer not contained in number
run_one W-M2 'b.contains(&t) || (t == "integer" && b.contains(&"number"))' 'b.contains(&t)' w_f02_integer_to_number_is_a_note judge_widen_test
# W-M3 any shared type counts as widening
run_one W-M3 "a.split('|').all(|t|" "a.split('|').any(|t|" w_f10_partial_overlap_is_violated judge_widen_test
# W-M4 widened note dropped: a silent HOLDS
run_one W-M4 'st.notes.push(format!(
                "retyped (widened)' 'drop(format!(
                "retyped (widened)' w_f01_nullable_widening_is_a_note judge_widen_test
# COUNTEREXAMPLE r0 (docs/falsification/COUNTEREXAMPLE-SPEC.md), in counterexample.rs
# CX-M1 base check skipped
run_one CX-M1 '        match check(base, base, &inst, 0) {
            Check::Valid => {}' '        match Check::Valid {
            Check::Valid => {}' cx_f11_unchecked_keyword_at_base_is_stated counterexample_test "$X"
# CX-M2 post check skipped
run_one CX-M2 '        match check(post, post, &inst, 0) {
            Check::Invalid => return Ok(inst),' '        match { let _ = post; Check::Invalid } {
            Check::Invalid => return Ok(inst),' cx_f12_unconfirmed_rejection_is_stated counterexample_test "$X"
# CX-M3 the kept enum value used instead of the lost one
run_one CX-M3 'a.into_iter().filter(|v| !b.contains(v)).map(Some).collect()' 'a.into_iter().filter(|v| b.contains(v)).map(Some).collect()' cx_f03_narrowed_carries_the_lost_value counterexample_test "$X"
# CX-M4 tightened value on the bound instead of past it
run_one CX-M4 '(true, Some(i)) => json!(if up { i + 1 } else { i - 1 }),' '(true, Some(i)) => json!(i),' cx_f04_tightened_steps_past_the_new_bound counterexample_test "$X"
# CX-M5 required properties not built (Amendment A: killed by cx_f09, not cx_f06)
run_one CX-M5 'for name in req.iter().map(String::as_str) {' 'for name in req.iter().map(String::as_str).take(0) {' cx_f09_b1_001_every_break_has_a_counterexample counterexample_test "$X"
# JUDGE-R3 r0 (docs/falsification/JUDGE-R3-SPEC.md)
# R3-M1 patternProperties check dropped
run_one R3-M1 'None if closed && patterned => out.push(stated_witness(&dotted, "not_evaluated", pv)),' 'None if false && closed && patterned => out.push(stated_witness(&dotted, "not_evaluated", pv)),' r3_f01_removal_under_pattern_properties_is_not_evaluated judge_r3_test
# R3-M2 finite domain not compared
run_one R3-M2 'match (pv.get("enum").is_none(), finite_domain(pv),' 'match (pv.get("enum").is_none() && false, finite_domain(pv),' r3_f03_enum_over_finite_domain_that_keeps_every_value_holds judge_r3_test
# R3-M3 allOf members ignored at base
run_one R3-M3 'let before = names(pre);' 'let before = own(pre);' r3_f07_required_moved_out_of_all_of_holds judge_r3_test
# R3-M4 allOf members ignored at post
run_one R3-M4 'names(post).into_iter()' 'own(post).into_iter()' r3_f08_newly_required_inside_all_of_is_escalated judge_r3_test
# COUNTEREXAMPLE r1 (docs/falsification/COUNTEREXAMPLE-R1-SPEC.md), in counterexample.rs
run_one K-M1 'let want = s.get("minProperties").and_then(Value::as_u64).unwrap_or(0) as usize;' 'let want = { let _ = s.get("minProperties"); 0usize };' k_f02_min_properties_padding counterexample_r1_test "$X"
run_one K-M2 'ok(if k == "minProperties" { o.len() as f64 >= b } else { o.len() as f64 <= b })' 'ok(if k == "minProperties" { true } else { o.len() as f64 <= b })' k_f03_min_properties_impossible counterexample_r1_test "$X"
run_one K-M3 'if s.get("uniqueItems") != Some(&Value::Bool(true)) {' 'if true || s.get("uniqueItems") != Some(&Value::Bool(true)) {' k_f06_unique_items_distinct_generation counterexample_r1_test "$X"
run_one K-M4 '(Some(true), Some(a)) => ok(!a.iter()' '(Some(true), Some(a)) => ok(true || !a.iter()' k_f07_unique_items_not_built_for_objects counterexample_r1_test "$X"
run_one K-M5 '        complete(base, base, &mut inst, 0)?;' '        if false { complete(base, base, &mut inst, 0)?; }' k_f08_dependent_required_completion counterexample_r1_test "$X"
run_one K-M6 'Some(o) => ok(deps.iter()' 'Some(o) => ok(true || deps.iter()' k_f09_dependent_required_unbuildable counterexample_r1_test "$X"
run_one K-M7 'ok((q - q.round()).abs() < 1e-9)' 'ok(true || (q - q.round()).abs() < 1e-9)' k_f11_multiple_of_is_read counterexample_r1_test "$X"
run_one K-M8 'v = (v / m).ceil() * m;' 'v += 0.0 * m;' k_f12_multiple_of_generation counterexample_r1_test "$X"
# COUNTEREXAMPLE r2 (docs/falsification/COUNTEREXAMPLE-R2-SPEC.md), in counterexample.rs
run_one C-M1 '"anyOf" => match (valid, first_unknown) {' '"anyOf" => match (0usize, Some("anyOf".to_string())) {' c_f02_any_of_read counterexample_r2_test "$X"
run_one C-M2 '(n, _) if n >= 2 => Check::Invalid,' '(n, _) if n >= 1 => Check::Valid,' c_f04_one_of_counted counterexample_r2_test "$X"
run_one C-M3 'Check::Invalid => Check::Valid,' 'Check::Invalid => Check::Invalid,' c_f05_not_read counterexample_r2_test "$X"
run_one C-M4 'req.extend(own(&r));' 'let _ = own(&r);' c_f01_all_of_required_built counterexample_r2_test "$X"
run_one C-M5 '"anyOf" => match (valid, first_unknown) {
                    (n, _) if n > 0 => Check::Valid,
                    (_, Some(r)) => Check::Unknown(r),' '"anyOf" => match (valid, first_unknown) {
                    (n, _) if n > 0 => Check::Valid,
                    (_, Some(r)) => Check::Unknown({ let _ = r; "anyOf".to_string() }),' c_f07_unknown_names_the_inner_keyword counterexample_r2_test "$X"
# JUDGE-D9 r0 (docs/falsification/JUDGE-D9-SPEC.md)
run_one D9-M1 '!values.is_empty() && values.iter().all(|v| value_has_type(v, &t))' '!values.is_empty() && { let _ = &t; true }' d9_f04_type_excluding_an_enum_value_is_violated judge_d9_test
run_one D9-M2 't.contains(&ty) || (ty == "integer" && t.contains(&"number"))' 't.contains(&ty)' d9_f06_integers_are_numbers judge_d9_test
run_one D9-M3 'let mut c: Vec<Value> = leaf.get("enum").and_then(Value::as_array).cloned().unwrap_or_default();' 'let mut c: Vec<Value> = Vec::new();' d9_f04_type_excluding_an_enum_value_is_violated judge_d9_test "$X"

# JUDGE-FILEPAIR r0 (docs/falsification/JUDGE-FILEPAIR-SPEC.md)
run_one FP-M1 'judge_one(head_path, BaseRead::Found(base_raw), Some(&head_raw))' 'judge_one(head_path, BaseRead::Found(head_raw.clone()), Some(&base_raw))' fp_f01_removed_field_violated_with_file_labels judge_filepair_test
run_one FP-M2 'let (c, b) = (head_path, base_path);' 'let (c, b) = (head_path, "main");' fp_f01_removed_field_violated_with_file_labels judge_filepair_test
run_one FP-M3 'let base_raw = std::fs::read(base_path).map_err(|e| anyhow::anyhow!("reading {base_path}: {e}"))?;' 'let base_raw = std::fs::read(head_path).map_err(|e| anyhow::anyhow!("reading {base_path}: {e}"))?;' fp_f10_unjudged_construct_is_not_evaluated judge_filepair_test
# JUDGE-REACH r0 (docs/falsification/JUDGE-REACH-SPEC.md, Amendments A and B)
run_one RE-M1 'diffs.iter().filter(|d| !covered_deep(d, &c))' 'diffs.iter().filter(|d| { let _ = d; false })' r_f04_unreferenced_definition_change_is_not_evaluated judge_reach_test
run_one RE-M2 'if va == vb || ANNOTATIONS.contains(&k.as_str()) || !KNOWN.contains(&k.as_str()) {' 'if va == vb {' r_f05b_annotation_beside_a_judged_change_is_not_unchecked judge_reach_test
run_one RE-M3 'diff(&format!("{here}/{}", escape(n)), Some(k), na.and_then(|m| m.get(n)), nb.and_then(|m| m.get(n)), out);' 'if !ANNOTATIONS.contains(&n.as_str()) { diff(&format!("{here}/{}", escape(n)), Some(k), na.and_then(|m| m.get(n)), nb.and_then(|m| m.get(n)), out); }' r_f08_property_named_like_an_annotation_counts judge_reach_test
run_one RE-M4 '        if c.whole.iter().any(|(_, w)| under(&d.ptr, w)) {' '        if c.whole.iter().any(|(_, w)| under(&d.ptr, w)) || c.kw.iter().any(|(_, p, _)| under(&d.ptr, p)) {' r_f01_change_behind_pattern_and_oneof_is_not_evaluated judge_reach_test
run_one RE-M5 'for kw in ["type", "enum", "pattern", "format"].iter()' 'for kw in ["type", "enum", "const", "pattern", "format"].iter()' r_f13_reached_node_unread_keyword_is_not_evaluated judge_reach_test
run_one RE-M6 '(None, Some(_)) => *adds_ok,' '(None, Some(_)) => { let _ = adds_ok; true }' r_f14_open_object_gains_constrained_property judge_reach_test
run_one RE-M7 'd.base == Some(Value::Bool(false)) && matches!(d.head, None | Some(Value::Bool(true)))' '{ let _ = &d.base; true }' r_f15_object_closed_is_not_evaluated judge_reach_test
run_one RE-M8 'reach::cover_kw(side, reach::owner(side, chain, "required"), "required");' 'reach::cover_kw(side, reach::owner(side, chain, "required"), "required"); reach::cover_kw(side, reach::owner(side, chain, "type"), "type");' r_f19_root_type_change_is_not_evaluated judge_reach_test
# JUDGE-DRAFT r0 (docs/falsification/JUDGE-DRAFT-SPEC.md)
run_one DR-M1 'c.draft = migration.is_some();' 'c.draft = { let _ = &migration; false };' d_f01_plain_migration_holds_and_is_stated judge_draft_test
run_one DR-M2 'if ident(pre) != ident(post) || !draft_neutral(pre, None) || !draft_neutral(post, None) {' 'if { let _ = ident; false } || !draft_neutral(pre, None) || !draft_neutral(post, None) {' d_f02_identifier_changed_is_not_covered judge_draft_test
run_one DR-M3 'if DRAFT_GUARDS.contains(&k.as_str()) {' 'if false && DRAFT_GUARDS.contains(&k.as_str()) {' d_f03_const_blocks_the_migration judge_draft_test
run_one DR-M4 'Value::Bool(_) => via.is_some_and(|k| DRAFT04_BOOLEANS.contains(&k)),' 'Value::Bool(_) => { let _ = via; false }' d_f07_draft04_booleans_do_not_block judge_draft_test
# JUDGE-ITEMS r0 (docs/falsification/JUDGE-ITEMS-SPEC.md)
run_one IT-M1 'if !judged(pv) || !judged(qv) || (pv.get("items").is_none() && qv.get("items").is_none()) {' 'if true || !judged(pv) || !judged(qv) || (pv.get("items").is_none() && qv.get("items").is_none()) {' i_f03_element_enum_added_is_a_break judge_items_test
run_one IT-M2 'let (pi, qi) = (pv.get("items").unwrap_or(&empty), qv.get("items").unwrap_or(&empty));' 'let (Some(pi), Some(qi)) = (pv.get("items"), qv.get("items")) else { let _ = &empty; return };' i_f04_items_added_is_a_break judge_items_test
run_one IT-M3 'let judged = |v: &Value| matches!(v.get("items"), None | Some(Value::Object(_))) && v.get("prefixItems").is_none() && admits_array(v);' 'let judged = |v: &Value| matches!(v.get("items"), None | Some(Value::Object(_)) | Some(Value::Array(_))) && v.get("prefixItems").is_none() && admits_array(v);
    let first = |v: &Value| v.get("items").and_then(|i| i.as_array().and_then(|a| a.first()).or(Some(i))).cloned();
    let (pv, qv) = (&{ let mut x = pv.clone(); if let Some(f) = first(pv) { x["items"] = f; } x }, &{ let mut x = qv.clone(); if let Some(f) = first(qv) { x["items"] = f; } x });' i_f08_tuple_items_stay_unchecked judge_items_test
run_one IT-M4 'Step::Item => Ok(Value::Array(Vec::new())),' 'Step::Item => Ok(Value::Object(Map::new())),' i_f06_element_property_retyped judge_items_test "$X"
# JUDGE-TESTS r0 and r1 (docs/falsification/JUDGE-TESTS-SPEC.md, JUDGE-TESTS-R1-SPEC.md), mutating action/tests.py
run_one TS-M1 '            witnesses.append(w)
        else:' '            passes += 1
        else:' t_f01_rejected_instance_is_a_break judge_tests_test "$P"
run_one TS-M2 'sys.exit(1 if witnesses else 4)' 'sys.exit(1 if witnesses else 0)' t_f02_passing_instances_are_not_evaluated judge_tests_test "$P"
run_one TS-M3 '        if kind(jb) == "invalid":
            not_promises += 1
            continue' '        if kind(jb) == "invalid":
            pass' t_f03_base_invalid_instance_is_not_a_promise judge_tests_test "$P"
run_one TS-M4 '            unreadable += 1
            not_checked.append({"file": name, "reason": why})
            continue' '            passes += 1
            continue' t_f06_unparsable_file_is_not_checked judge_tests_test "$P"
run_one TS-M5 '        ah = ajv[name]["head"] if ajv is not None else None' '        ah = jh if ajv is not None else None' t_f16_validators_disagree_is_disputed judge_tests_test "$P"
run_one TS-M6 '        if ah is not None and kind(jh) != kind(ah):
            disputed.append({"file": name, "side": "head", "jsonschema": kind(jh), "ajv": kind(ah)})
            continue' '        if ah is not None and kind(jh) != kind(ah):
            witnesses.append({"file": name, "pointer": "", "message": "split", "ajv_message": "split"})
            continue' t_f16_validators_disagree_is_disputed judge_tests_test "$P"
run_one TS-M7 '        if ah is not None:
            w["ajv_message"] = ah["invalid"]["message"]
            witnesses.append(w)
        else:
            unconfirmed.append(w)' '        if ah is not None:
            w["ajv_message"] = ah["invalid"]["message"]
        witnesses.append(w)' t_f17_single_validator_is_unconfirmed judge_tests_test "$P"

# CN-M1 the normal form is never applied (JUDGE-CANON r0)
run_one CN-M1 '        a.is_object() && b.is_object() && normal(a) == normal(b)
    }' '        a.is_object() && b.is_object() && form(a, false) == form(b, false)
    }' c_f01_type_list_as_anyof_holds_and_is_stated judge_canon_test
# CN-M2 a typed keyword is shared instead of following its type
run_one CN-M2 '    fn is_typed(k: &str) -> bool {
        TYPED_STRING.contains(&k) || TYPED_NUMBER.contains(&k) || TYPED_ARRAY.contains(&k) || TYPED_OBJECT.contains(&k)
    }' '    fn is_typed(_k: &str) -> bool {
        false
    }' c_f02_typed_keyword_follows_its_type judge_canon_test
# CN-M3 annotations are compared instead of set aside
run_one CN-M3 '                    .filter(|(k, _)| !ANNOTATIONS.contains(&k.as_str()) && KNOWN.contains(&k.as_str()))
                    .map(|(k, x)| {
                        let kw = k.as_str();' '                    .filter(|(k, _)| ANNOTATIONS.contains(&k.as_str()) || KNOWN.contains(&k.as_str()))
                    .map(|(k, x)| {
                        let kw = k.as_str();' c_f06_annotations_are_set_aside judge_canon_test
# CN-M4 property names filtered as if they were keywords (Amendment A)
run_one CN-M4 '                            Value::Object(names) if NAME_MAPS.contains(&kw) || kw == "dependencies" => {
                                Value::Object(names.iter().map(|(n, s)| (n.clone(), if s.is_object() { sub(s) } else { s.clone() })).collect())
                            }' '                            Value::Object(names) if NAME_MAPS.contains(&kw) || kw == "dependencies" => {
                                Value::Object(names.iter().filter(|(n, _)| KNOWN.contains(&n.as_str())).map(|(n, s)| (n.clone(), if s.is_object() { sub(s) } else { s.clone() })).collect())
                            }' c_f11_change_under_a_property_name_is_not_hidden judge_canon_test

# CR1-M1 const not rewritten to enum (JUDGE-CANON r1)
run_one CR1-M1 '        if let Some(c) = kept.remove("const") {' '        if let Some(c) = kept.remove("const_never") {' c1_f01_enum_of_one_value_to_const_holds judge_canon_r1_test
# CR1-M2 boolean is not a literal
run_one CR1-M2 '                Some("boolean") => return bare(vec![Value::Bool(false), Value::Bool(true)]),' '                Some("boolean_never") => return bare(vec![Value::Bool(false), Value::Bool(true)]),' c1_f03_enum_to_anyof_of_boolean_and_typed_const_holds judge_canon_r1_test
# CR1-M3 oneOf union without the disjointness check
run_one CR1-M3 '                        if kw == "anyOf" || disjoint {' '                        if kw == "anyOf" || disjoint || true {' c1_f07_overlapping_oneof_is_not_the_union judge_canon_r1_test
# CR1-M4 the type constraint is not folded into the enum
run_one CR1-M4 '                Some(Value::String(t)) => {
                    types.push(t.clone());
                    drop = Some("type");
                }' '                Some(Value::String(_t)) => {}' c1_f09_value_outside_the_type_is_dropped judge_canon_r1_test
# CR1-M5 D1 keeps refusing an annotation-only equality
run_one CR1-M5 '        a.is_object() && b.is_object() && normal(a) == normal(b)
    }' '        a.is_object() && b.is_object() && form(a, false) != form(b, false) && normal(a) == normal(b)
    }' c1_f08_annotation_only_difference_at_a_combinator_holds_silently judge_canon_r1_test
cp "$J.orig" "$J"; rm "$J.orig"; cp "$X.orig" "$X"; rm "$X.orig"; cp "$P.orig" "$P"; rm "$P.orig"
cargo build -q 2>/dev/null
echo "mutants killed: $pass of $((pass+fail))"
[ "$fail" -eq 0 ]
