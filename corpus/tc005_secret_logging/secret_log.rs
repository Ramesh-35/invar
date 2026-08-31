// TC-005 — cleartext secret logging (Rust half).
//
// Not cross-repository. catch_signature: "tc005". A local control the composite
// is expected to catch.

pub fn record_login(user: &str, auth_token: &str) {
    // VIOLATION: prints a secret in cleartext.
    println!("login user={user} auth_token={auth_token}");
}
