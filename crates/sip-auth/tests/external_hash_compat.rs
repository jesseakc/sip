//! Prove that a password hash generated *outside* SIP is accepted by SIP.
//!
//! The demo environment generates its accounts' argon2id hashes with a Python
//! tool rather than through the API, because the first administrator of a new
//! tenant has to exist before any authenticated call can create users. That
//! only works if SIP's `verify_password` accepts them.
//!
//! `Argon2::default()` (argon2 0.5) is Argon2id, v19, m=19456, t=2, p=1, 32-byte
//! output. Verification reads the cost parameters out of the encoded hash, so a
//! hash produced with those parameters must verify.
//!
//! Run explicitly:
//!
//! ```text
//! SIP_TEST_PASSWORD=<password> \
//! SIP_TEST_PASSWORD_HASH='$argon2id$v=19$m=19456,t=2,p=1$...' \
//!   cargo test -p sip-auth --test external_hash_compat -- --nocapture
//! ```
//!
//! Marked `#[ignore]` so it does not run (and does not report a vacuous pass) in
//! a normal `cargo test`, which has no password or hash to check. Invoke it with
//! `-- --ignored` as shown above. No credential literal is ever committed.

use std::env;

use sip_auth::{hash_password, verify_password};

const PASSWORD_VAR: &str = "SIP_TEST_PASSWORD";
const HASH_VAR: &str = "SIP_TEST_PASSWORD_HASH";

#[test]
#[ignore = "needs SIP_TEST_PASSWORD and SIP_TEST_PASSWORD_HASH; run with -- --ignored"]
fn externally_generated_hash_is_accepted() {
    let (Ok(password), Ok(hash)) = (env::var(PASSWORD_VAR), env::var(HASH_VAR)) else {
        eprintln!(
            "skipping: set {PASSWORD_VAR} and {HASH_VAR} to check an externally generated hash"
        );
        return;
    };

    match verify_password(&password, &hash) {
        Ok(true) => {}
        Ok(false) => panic!(
            "SIP rejected a hash generated for the supplied password. \
             The generator's argon2 parameters do not match Argon2::default() \
             (Argon2id v19, m=19456, t=2, p=1)."
        ),
        Err(e) => panic!("SIP could not parse the supplied hash: {e}"),
    }

    // And a wrong password must still be rejected, so the acceptance above is
    // not an artefact of a permissive parser.
    assert!(
        !verify_password("definitely-not-the-password", &hash).expect("verify should parse"),
        "a mismatched password verified successfully"
    );

    // Finally, confirm the parameters in the hash are the ones SIP itself emits,
    // so a generated hash is indistinguishable from a native one.
    let native = hash_password(&password).expect("hashing should succeed");
    let params_of = |h: &str| {
        h.split('$')
            .filter(|s| s.contains("m="))
            .collect::<Vec<_>>()
            .join("")
    };
    assert_eq!(
        params_of(&hash),
        params_of(&native),
        "generated hash uses different cost parameters than SIP's own hashes"
    );

    eprintln!("OK: externally generated hash accepted, parameters match native output");
}
