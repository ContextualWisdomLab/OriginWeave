use originweave_bap::BapRecoveryEvidenceDigest;

const EVIDENCE_SHA256: [u8; 32] = [
    0xcd, 0xc7, 0xdc, 0x8d, 0x4a, 0x35, 0xad, 0xfc, 0x9e, 0x13, 0x51, 0xad, 0x81, 0x82, 0x1c, 0xf7,
    0x17, 0xb7, 0x87, 0xf8, 0x9b, 0x6e, 0xfb, 0xe7, 0x25, 0x6f, 0x4a, 0xaf, 0x8e, 0xcc, 0x6c, 0x62,
];
const EXPECTED_DIGEST: &str =
    "sha256:cdc7dc8d4a35adfc9e1351ad81821cf717b787f89b6efbe7256f4aaf8ecc6c62";

#[test]
fn recovery_evidence_digest_accepts_exact_sha256_output_without_owning_evidence_hashing() {
    let digest = BapRecoveryEvidenceDigest::from_sha256_bytes(EVIDENCE_SHA256);
    assert_eq!(digest.as_str(), EXPECTED_DIGEST);

    let parsed = BapRecoveryEvidenceDigest::parse(EXPECTED_DIGEST);
    assert!(parsed.is_ok(), "{parsed:?}");
    let Ok(parsed) = parsed else {
        unreachable!("asserted canonical derived digest")
    };
    assert_eq!(digest, parsed);
}
