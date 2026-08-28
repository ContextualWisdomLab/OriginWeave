use originweave_bap::BapRecoveryEvidenceDigest;

const EVIDENCE: &[u8] = b"originweave recovery evidence";
const EXPECTED_DIGEST: &str =
    "sha256:cdc7dc8d4a35adfc9e1351ad81821cf717b787f89b6efbe7256f4aaf8ecc6c62";

#[test]
fn recovery_evidence_digest_hashes_and_verifies_exact_bytes() {
    let digest = BapRecoveryEvidenceDigest::from_bytes(EVIDENCE);
    assert_eq!(digest.as_str(), EXPECTED_DIGEST);
    assert!(digest.matches_bytes(EVIDENCE));
    assert!(!digest.matches_bytes(b"originweave recovery evidence\n"));

    let parsed = BapRecoveryEvidenceDigest::parse(EXPECTED_DIGEST);
    assert!(parsed.is_ok(), "{parsed:?}");
    let Ok(parsed) = parsed else {
        unreachable!("asserted canonical derived digest")
    };
    assert_eq!(digest, parsed);
}
