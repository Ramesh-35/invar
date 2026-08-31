// TC-005 — cleartext secret logging (TypeScript half).
//
// Not cross-repository. catch_signature: "tc005". This is exactly the kind of
// LOCAL, single-file defect that a good static rule SHOULD catch — it exists in
// the corpus as a control: the composite is expected to isolate it
// (CAUGHT_BY_COMPOSITE). If the composite misses this, something is wrong with
// the composite, not with INVAR's thesis.

export function authenticate(username: string, password: string): void {
  // VIOLATION: logs a secret in cleartext.
  console.log(`auth attempt user=${username} password=${password}`);
}
