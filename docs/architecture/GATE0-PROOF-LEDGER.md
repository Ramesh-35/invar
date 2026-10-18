# GATE0-PROOF-LEDGER: proof engine hash baseline

Taken 2026-10-18 at the clean tree of commit 8861f63a65dfbc1f94fef5bf8de6ce4cca45ff96 (slice 2b REV1
landed). Authorized by the ruling of 2026-10-18 (I1 to I3 closed, Gate 0
initialized). Full SHA-256. A mismatch against this ledger at any later gate of
the INVAR-PROOF-001 arc stops the arc for ruling.

## Class 1 (must stay byte-identical through the proof arc)

```
71b521ce7528daee7a4314dc5edb40c421bb24f1a88e1c9660fd1305cca94c72  crates/invar-falsify/src/surface.rs
df9ef6e87de411df7157981219fd71e123acbb03b25540d2498b0e7aace080ea  crates/invar-falsify/src/backtest.rs
49972028d11b0a53679d29bd8ecfd1abcadb7bc83fb8c04881b30201b2764f27  crates/invar-falsify/src/invariant.rs
1254fde8cdd6f64a4fcc33d223af90b0ba34341f0cf3b03a38e2a26956fc9b9b  crates/invar-falsify/src/yaml.rs
004a26dec58e6746f9a36e68bd83cde6dabc813c1b7bbb01dcaeb26e9d0ed7aa  crates/invar-falsify/src/codepairs.rs
a8023ba46533059c243283fb628e95a318dd8ea3c641f7bf1989a1b66144c867  crates/invar-falsify/Cargo.toml
7c5f4cee5d13c0c43a0e22db0fac95dcf33f25974ab33fb6b08d35094ffdd51b  Cargo.toml
81bcb0ff4e122baaf3ee411fd7f56b7fa976e62e73512f401d1141c73993d771  Cargo.lock
090d8574fb2f30f2edaa5ab5f9fae7e9722d34fba9e4d2b7c09a9f2b4e5b8ffc  docs/falsification/experiment-c/DEFINITION.md
02d0c4299aa47e09e363bd44985c482be5ca0703332994b8f59dbdd802e9be25  docs/falsification/experiment-c/EVALUATION-PROTOCOL.md
00daf71769bb0e0ea767bf2fddfde762c4db3b227a3b067948d7c48d39f43dfa  docs/falsification/experiment-c/GROUND-TRUTH-REPORT.md
6c3982b2105f76a345737fd37c331e32380aca4612e872e22d8fc6571161d236  docs/falsification/experiment-c/corpus.tsv
8b8d1b9002c7dda3414e5f0b461f7022d5deb6730d4fac68c8cb3087b79a559b  docs/falsification/experiment-c/labels.tsv
```

## Class 3 pre-change baselines (granted to change by D9 and I1b; recorded so
every later diff is attributable)

```
d30f5e4bf6b3ede55a4853b5c082e5b4a79452d7eaa2f3f39658d91708ca4a3e  crates/invar-falsify/src/config.rs
357d26c28503bcc5fe0879f6a84e2506531495e4000d20309f77fcc502b10347  crates/invar-falsify/src/lock.rs
0dcbe09af5f84d345f78bb411927b5fac1c1f21a2ea31d4c3861491e22bc70c3  crates/invar-falsify/src/check.rs
7b4768b0543f603400eb1e99e462e96510e032bd97a767bc853d3cedd8785017  crates/invar-falsify/src/main.rs
```

## Landed slices 1, 2, 2b and REV1 (byte-identical except lib.rs, Class 4 for
module registration)

```
42e158a9e22f619980750d840a9cb091c524b7e366bd774c50db1a63e1e8ddfa  crates/invar-falsify/src/valuepairs.rs
07e91fae4374f70bba92cfd8bd6b0ed3380df6a79413657b01eda0d851fc9426  crates/invar-falsify/src/valuepairs/strings.rs
7b8a8a80b05e93b277fe3a328db3f3ba74c65cc385d5d6e4cf7d92d4f0c99b07  crates/invar-falsify/src/init.rs
af72c0c7d1d5c45ef09d7fbeda4e847239cd9b231042841190c6b5c93b3a268f  crates/invar-falsify/src/lib.rs
d53878b033a86d6d66b93109b77e336ba22b7d374fdb1f05f41d27f442b4a9c7  crates/invar-falsify/tests/harness_test.rs
6a77cf8963b48f44b592394a01ee06b10af036c0cac408b3016893a41138213e  crates/invar-falsify/tests/init_slice1_test.rs
5d55a85e1c63dc271bf6b0c054928e26ad2d536e4d6f7bff0573c9444ab2c794  crates/invar-falsify/tests/init_slice2_test.rs
86df1a0ac83c4dd9fa90d4744566b30439c15215b545341ab4d5028a99595913  crates/invar-falsify/tests/init_slice2b_test.rs
850ebf898edfd2f6aeb51308684c06a9d0e8e56188a3e5ef4b65cf7ad59da848  crates/invar-falsify/tests/init_slice2b_rev1_test.rs
```
