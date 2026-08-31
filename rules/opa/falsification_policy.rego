# INVAR falsification — LOCAL, EXPERIMENTAL OPA policy.
#
# This is NOT comprehensive enterprise policy. Phase 1 does not yet wire an
# input document into this policy, so the OPA leg of the composite reports
# SKIPPED (never PASS). This file declares the intended tc005 cleartext-secret
# check so the policy leg has a real artifact to grow from.
package invar.falsification

default allow := true

# tc005: deny when any observed log statement contains a cleartext secret.
deny contains msg if {
    some entry in input.log_statements
    regex.match(`(?i)(password|secret|auth_token)`, entry.text)
    msg := sprintf("tc005: cleartext secret in log statement at %v", [entry.location])
}
