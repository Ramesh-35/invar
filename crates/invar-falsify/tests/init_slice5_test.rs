//! INIT-DISCOVERY-SLICE5 fixtures (rulings S5-D1 to S5-D6): receiver
//! verification. A field read counts for a shape only through a receiver
//! declared as that shape in the consumer file (annotation, cast, explicit
//! type argument, a local function's declared return type, or an element of
//! a declared array); writes never count. A shape that passes file-wide but
//! not on declared receivers abstains RECV, never silently. Each fixture is
//! a two-package workspace (package `a` produces Order, `b` consumes), run
//! through `invar init --stdout` twice, byte-identical.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT_AB: (&str, &str) = ("package.json", r#"{"private":true,"workspaces":["a","b"]}"#);
const PKG_A: (&str, &str) = ("a/package.json", r#"{"name":"a"}"#);
const PKG_B: (&str, &str) = ("b/package.json", r#"{"name":"b"}"#);
const ORDER: &str = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n}\n";
const ONE: &str = "SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE";
const LOUD: &str = "SUMMARY packages=2 candidates=1 proposals=0 abstentions=1 inventory=0 status=COMPLETE";
const STATUS: &str = "status: PROPOSED (heuristic; each counted read is on a receiver declared as the shape in this file; field writes excluded; types are not inferred)";

fn ws(tag: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("invar_s5_{tag}_{}_{nanos}", std::process::id()));
    for (rel, body) in files {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }
    dir
}

fn run(dir: &Path) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_invar")).args(["init", "--stdout", "--root"]).arg(dir).output().unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

struct Out {
    proposals: usize,
    e5: Vec<String>,
    abstain: Vec<String>,
    summary: String,
    raw: String,
}

fn init(dir: &Path) -> Out {
    let (a, b) = (run(dir), run(dir));
    assert_eq!(a, b, "D01: init output is not deterministic");
    Out {
        proposals: a.lines().filter(|l| *l == "[[invariant]]").count(),
        e5: a.lines().filter_map(|l| l.strip_prefix("# E5 ")).filter(|l| l.contains(" fields read: ")).map(String::from).collect(),
        abstain: a.lines().filter_map(|l| l.strip_prefix("# ABSTAIN ")).map(|l| format!("ABSTAIN {l}")).collect(),
        summary: a.lines().last().unwrap_or("").trim_start_matches("# ").to_string(),
        raw: a,
    }
}

/// `a/order.ts` declares Order; `b/src/x.ts` is `consumer`.
fn case(tag: &str, consumer: &str) -> Out {
    init(&ws(tag, &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", ORDER), ("b/src/x.ts", consumer)]))
}

fn proposed(o: &Out, e5: &str) {
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert!(o.abstain.is_empty(), "{}", o.raw);
    assert_eq!(o.e5, vec![e5.to_string()], "{}", o.raw);
    assert!(o.raw.contains(&format!("# {STATUS}\n")), "{}", o.raw);
    assert_eq!(o.summary, ONE, "{}", o.raw);
}

fn recv(o: &Out, line: &str) {
    assert_eq!(o.proposals, 0, "{}", o.raw);
    assert_eq!(o.abstain, vec![line.to_string()], "{}", o.raw);
    assert_eq!(o.summary, LOUD, "{}", o.raw);
}

fn e5(lines: (usize, usize)) -> String {
    format!("Order fields read: currency_code (b/src/x.ts:{}), total_minor (b/src/x.ts:{})", lines.0, lines.1)
}

fn recv_line(accessed: usize, verified: usize) -> String {
    format!(
        "ABSTAIN RECV b/src/x.ts:1 Order: {accessed} of its fields are accessed in this file, {verified} read on a receiver declared as Order; the others are writes or are read off values init cannot tie to Order; not guessed"
    )
}

// ------------------------------------------------------- declared (R-S5-1)

#[test]
fn s5_f1_an_annotated_parameter_proposes() {
    let o = case("f1", "import { Order } from 'a/order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n");
    proposed(&o, &e5((2, 2)));
}

#[test]
fn s5_f2_an_untyped_receiver_abstains_recv() {
    // The slice 1 n10 workspace (S5-D3): the reads are on `other`, not on
    // the value declared as Order.
    let o = case("f2", "import { Order } from 'a/order';\nexport const f = (o: Order, other: any) => [other.total_minor, other.currency_code];\n");
    recv(&o, &recv_line(2, 0));
}

#[test]
fn s5_f4_a_cast_proposes() {
    let o = case("f4", "import { Order } from 'a/order';\nexport function f(x: unknown) {\n  const o = x as Order;\n  return [o.total_minor, o.currency_code];\n}\n");
    proposed(&o, &e5((4, 4)));
}

#[test]
fn s5_f5_an_explicit_type_argument_proposes() {
    let o = case(
        "f5",
        "import { Order } from 'a/order';\ndeclare function useState<T>(v: T): [T, (v: T) => void];\nexport function f() {\n  const [o, setO] = useState<Order | null>(null);\n  return [o?.total_minor, o?.currency_code, setO];\n}\n",
    );
    proposed(&o, &e5((5, 5)));
}

#[test]
fn s5_f6_a_local_return_type_proposes() {
    let o = case(
        "f6",
        "import { Order } from 'a/order';\nasync function load(): Promise<Order> {\n  return fetchIt();\n}\nexport async function f() {\n  const o = await load();\n  return [o.total_minor, o.currency_code];\n}\n",
    );
    proposed(&o, &e5((7, 7)));
}

#[test]
fn s5_f7_a_for_of_element_proposes() {
    let o = case(
        "f7",
        "import { Order } from 'a/order';\nexport function f(list: Order[]) {\n  for (const o of list) {\n    use(o.total_minor, o.currency_code);\n  }\n}\n",
    );
    proposed(&o, &e5((4, 4)));
}

#[test]
fn s5_f8_array_callbacks_and_indexing_propose() {
    let o = case(
        "f8",
        "import { Order } from 'a/order';\nexport function f(list: Array<Order>) {\n  const totals = list.map((o) => o.total_minor);\n  return [totals, list[0].currency_code];\n}\n",
    );
    proposed(&o, &e5((4, 3)));
}

#[test]
fn s5_f10_a_this_field_proposes() {
    let o = case(
        "f10",
        "import { Order } from 'a/order';\nexport class C {\n  private order: Order;\n  f() {\n    return [this.order.total_minor, this.order.currency_code];\n  }\n}\n",
    );
    proposed(&o, &e5((5, 5)));
}

// --------------------------------------------------------- refused (R-S5-2)

#[test]
fn s5_f3_writes_never_count() {
    let o = case(
        "f3",
        "import { Order } from 'a/order';\nexport function f(o: Order) {\n  o.total_minor = 1;\n  o.currency_code += \"x\";\n}\n",
    );
    recv(&o, &recv_line(2, 0));
}

#[test]
fn s5_f9_a_type_argument_of_another_type_is_not_the_shape() {
    let o = case(
        "f9",
        "import { Order } from 'a/order';\ninterface Box<T> {\n  total_minor: number;\n  currency_code: string;\n  inner: T;\n}\nexport const f = (x: Box<Order>) => [x.total_minor, x.currency_code];\n",
    );
    recv(&o, &recv_line(2, 0));
}

// ---------------------------------------------------- restriction (R-S5-4)

#[test]
fn s5_r1_a_pair_keeps_only_its_verified_shape() {
    // Two shapes imported; reads on Order are declared, reads on Invoice
    // are not. The pair keeps Order; Invoice abstains RECV.
    let shapes = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n}\nexport interface Invoice {\n  due_date: string;\n  amount_due: number;\n}\n";
    let x = "import { Order, Invoice } from 'a/order';\nexport const f = (o: Order, i: any) => [o.total_minor, o.currency_code, i.due_date, i.amount_due];\n";
    let o = init(&ws("r1", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", shapes), ("b/src/x.ts", x)]));
    assert_eq!(o.proposals, 1, "{}", o.raw);
    assert_eq!(o.e5, vec![e5((2, 2))], "{}", o.raw);
    assert_eq!(
        o.abstain,
        vec!["ABSTAIN RECV b/src/x.ts:1 Invoice: 2 of its fields are accessed in this file, 0 read on a receiver declared as Invoice; the others are writes or are read off values init cannot tie to Invoice; not guessed".to_string()],
        "{}",
        o.raw
    );
    assert_eq!(o.summary, "SUMMARY packages=2 candidates=2 proposals=1 abstentions=1 inventory=0 status=COMPLETE", "{}", o.raw);
}

#[test]
fn s5_r2_an_untyped_read_never_joins_the_evidence() {
    // Three fields read, two on the declared receiver: E5 lists those two.
    let shape = "export interface Order {\n  total_minor: number;\n  currency_code: string;\n  placed_at: string;\n}\n";
    let x = "import { Order } from 'a/order';\nexport const f = (o: Order, other: any) => [o.total_minor, o.currency_code, other.placed_at];\n";
    let o = init(&ws("r2", &[ROOT_AB, PKG_A, PKG_B, ("a/order.ts", shape), ("b/src/x.ts", x)]));
    proposed(&o, &e5((2, 2)));
}

// ------------------------------------------------------ no regression (N1)

#[test]
fn s5_n1_a_typed_proposal_changes_only_the_s5_d4_text() {
    // Golden recorded at Gate 1 on the SLICE4 binary (aa03983 plus the SLICE4
    // patch); SLICE5 may change only the header sentence and the status line.
    let x = "import { Order } from 'a/order';\nexport const f = (o: Order) => [o.total_minor, o.currency_code];\n";
    let o = case("n1", x);
    let want = N1_SLICE4
        .replace(
            "# direct fields read there. The receiver of those reads is NOT verified.\n",
            "# direct fields read there, each on a receiver declared as that shape in\n# the consumer file (field writes excluded; types are not inferred).\n",
        )
        .replace("# status: PROPOSED (heuristic; receiver of field reads not verified)\n", &format!("# {STATUS}\n"))
        // S7-D6 (INIT-DISCOVERY-SLICE7): the header sentence, and nothing else.
        .replace(
            "# workspace package, imported by name in another, with at least 2 of its\n# direct fields read there, each on a receiver declared as that shape in\n# the consumer file (field writes excluded; types are not inferred).\n",
            "# workspace package, with at least 2 of its direct fields read in a file of\n# another, each on a receiver declared as that shape in that file or\n# assigned from a function imported from the shape's package whose declared\n# return type is the shape (field writes excluded; types are not inferred).\n",
        )
        // S8-D3 (INIT-DISCOVERY-SLICE8): the header sentence, and nothing else.
        .replace(
            "# another, each on a receiver declared as that shape in that file or\n# assigned from a function imported from the shape's package whose declared\n# return type is the shape (field writes excluded; types are not inferred).\n",
            "# another, each on a receiver declared as that shape in that file, assigned\n# from a function imported from the shape's package whose declared return\n# type is the shape, or read through a property, at most two hops from a\n# receiver typed in that file, whose declared type is the shape (field\n# writes excluded; types are not inferred).\n",
        );
    assert_ne!(want, N1_SLICE4);
    assert_eq!(o.raw, want);
}

// ------------------------------------------------------- determinism (M1)

#[test]
fn s5_m1_declaration_order_does_not_matter() {
    // init() asserts two runs byte-identical; declaring the untyped value
    // before or after the typed one gives the same evidence.
    let a = case("m1a", "import { Order } from 'a/order';\nexport const f = (o: Order, other: any) => [o.total_minor, other.x, o.currency_code];\n");
    let b = case("m1b", "import { Order } from 'a/order';\nexport const f = (other: any, o: Order) => [o.total_minor, other.x, o.currency_code];\n");
    assert_eq!(a.e5, b.e5);
    proposed(&a, &e5((2, 2)));
}

const N1_SLICE4: &str = r#"# invar.toml generated by `invar init`.
#
# HONEST SCOPE: discovery is HEURISTIC (text patterns, not a parser). Each
# [[invariant]] block below is a PROPOSAL: a TypeScript shape exported by one
# workspace package, imported by name in another, with at least 2 of its
# direct fields read there. The receiver of those reads is NOT verified.
# Review every block before trusting a verdict. Schema files are inventory
# only. ABSTAIN lines are refusals to propose, not negative findings. Paths
# are relative to this file.

[discovery]
files_scanned = "5"
truncated = "false"
packages = "2"
candidates = "1"
contracts_proposed = "1"
abstentions = "0"
inventory = "0"

# PROPOSED code:order.ts->x.ts
# status: PROPOSED (heuristic; receiver of field reads not verified)
# E1 producer package a at a/package.json (package.json "workspaces" entry "a")
# E2 Order declared at a/order.ts:1
# E3 consumer package b at b/package.json (package.json "workspaces" entry "b")
# E4 Order imported at b/src/x.ts:1 from "a/order", resolved to a/order.ts
# E5 Order fields read: currency_code (b/src/x.ts:2), total_minor (b/src/x.ts:2)
# E5 Order stoplisted reads: (none)
# E6 pair a/order.ts -> b/src/x.ts
# category: production
# lock: run `invar lock` before `invar check`. Without a lock, check evaluates only payload.<field> reads and says nothing about this proposal (a HOLDS would be vacuous). `invar lock` binds every field of the producer file that the consumer mentions in any form, which is wider than the evidence above.
[[invariant]]
kind = "schema"
id = "code:order.ts->x.ts"
producer = "a/order.ts"
consumer = "b/src/x.ts"
evidence = "Order currency_code total_minor"

# ---- Abstentions (init declined to propose; not negative findings) ----
# (none)
# ---- Found but NOT proposed (inventory only) ----
# (none)
# SUMMARY packages=2 candidates=1 proposals=1 abstentions=0 inventory=0 status=COMPLETE
"#;
