use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const SOURCE_REVISION_ENV: &str = "ORIGINWEAVE_PERFORMANCE_SOURCE_REVISION";
const RECEIPT_PATH_ENV: &str = "ORIGINWEAVE_PERFORMANCE_RECEIPT_PATH";
const EVIDENCE_DIR_ENV: &str = "ORIGINWEAVE_PERFORMANCE_EVIDENCE_DIR";
const SELECTED_PROFILE: &str = "stacked-content-coding";
const RESULT_FILENAME: &str = "content-coding-result.json";
const RUNTIME_FILENAME: &str = "content-coding-runtime.json";
const FIXTURE_FILENAME: &str = "content-coding-fixture.json";
const SAMPLE_COUNT: usize = 31;
const P95_BUDGET_MICROSECONDS: u128 = 20_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Receipt {
    source_revision: String,
    source_revision_source: String,
    environment_id: String,
    runtime_os: String,
    runtime_arch: String,
    runtime_parallelism: usize,
    network_authority: String,
    network_acceptance_status: String,
    evidence_authority: String,
    evidence_acceptance_status: String,
    profile: String,
    decoded_bytes: usize,
    coded_bytes: usize,
    samples: usize,
    p50_microseconds: u128,
    p95_microseconds: u128,
    maximum_microseconds: u128,
    budget_microseconds: u128,
    budget_status: String,
    source_acceptance_status: String,
    acceptance_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceDigests {
    result_sha256: String,
    runtime_sha256: String,
    fixture_sha256: String,
}

fn parse_candidate_sha(value: &str) -> Result<String, String> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    {
        return Err("candidate SHA must be an exact lowercase 40-hex Git object id".to_owned());
    }
    Ok(value.to_owned())
}

fn safe_token(value: &str, label: &str) -> Result<String, String> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b':' | b'/' | b'@' | b'+' | b'-')
        })
    {
        return Err(format!("{label} must be one bounded receipt token"));
    }
    Ok(value.to_owned())
}

fn parse_usize(value: String, label: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("{label} is not a valid usize: {error}"))
}

fn parse_u128(value: String, label: &str) -> Result<u128, String> {
    value
        .parse::<u128>()
        .map_err(|error| format!("{label} is not a valid u128: {error}"))
}

fn take(fields: &mut BTreeMap<String, String>, name: &str) -> Result<String, String> {
    fields
        .remove(name)
        .ok_or_else(|| format!("performance receipt omitted {name}"))
}

fn parse_receipt_line(line: &str) -> Result<Receipt, String> {
    let mut fields = BTreeMap::new();
    for token in line.split_ascii_whitespace() {
        let (name, value) = token
            .split_once('=')
            .ok_or_else(|| "performance receipt token omitted '='".to_owned())?;
        if name.is_empty() || value.is_empty() {
            return Err("performance receipt token must contain name and value".to_owned());
        }
        if fields.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err(format!("performance receipt duplicated field {name}"));
        }
    }

    let receipt = Receipt {
        source_revision: parse_candidate_sha(&take(&mut fields, "source_revision")?)?,
        source_revision_source: safe_token(
            &take(&mut fields, "source_revision_source")?,
            "source_revision_source",
        )?,
        environment_id: safe_token(&take(&mut fields, "environment_id")?, "environment_id")?,
        runtime_os: safe_token(&take(&mut fields, "runtime_os")?, "runtime_os")?,
        runtime_arch: safe_token(&take(&mut fields, "runtime_arch")?, "runtime_arch")?,
        runtime_parallelism: parse_usize(
            take(&mut fields, "runtime_parallelism")?,
            "runtime_parallelism",
        )?,
        network_authority: safe_token(
            &take(&mut fields, "network_authority")?,
            "network_authority",
        )?,
        network_acceptance_status: safe_token(
            &take(&mut fields, "network_acceptance_status")?,
            "network_acceptance_status",
        )?,
        evidence_authority: safe_token(
            &take(&mut fields, "evidence_authority")?,
            "evidence_authority",
        )?,
        evidence_acceptance_status: safe_token(
            &take(&mut fields, "evidence_acceptance_status")?,
            "evidence_acceptance_status",
        )?,
        profile: safe_token(&take(&mut fields, "profile")?, "profile")?,
        decoded_bytes: parse_usize(take(&mut fields, "decoded_bytes")?, "decoded_bytes")?,
        coded_bytes: parse_usize(take(&mut fields, "coded_bytes")?, "coded_bytes")?,
        samples: parse_usize(take(&mut fields, "samples")?, "samples")?,
        p50_microseconds: parse_u128(take(&mut fields, "p50_us")?, "p50_us")?,
        p95_microseconds: parse_u128(take(&mut fields, "p95_us")?, "p95_us")?,
        maximum_microseconds: parse_u128(take(&mut fields, "max_us")?, "max_us")?,
        budget_microseconds: parse_u128(take(&mut fields, "budget_us")?, "budget_us")?,
        budget_status: safe_token(&take(&mut fields, "budget_status")?, "budget_status")?,
        source_acceptance_status: safe_token(
            &take(&mut fields, "source_acceptance_status")?,
            "source_acceptance_status",
        )?,
        acceptance_status: safe_token(
            &take(&mut fields, "acceptance_status")?,
            "acceptance_status",
        )?,
    };
    if !fields.is_empty() {
        return Err(format!(
            "performance receipt contained unexpected fields: {}",
            fields.keys().cloned().collect::<Vec<_>>().join(",")
        ));
    }
    Ok(receipt)
}

fn profile_rank(profile: &str) -> Option<usize> {
    match profile {
        "gzip-deflate-256k" => Some(0),
        "deflate-gzip-1m" => Some(1),
        _ => None,
    }
}

fn expected_decoded_bytes(profile: &str) -> Option<usize> {
    match profile {
        "gzip-deflate-256k" => Some(256 * 1024),
        "deflate-gzip-1m" => Some(1024 * 1024),
        _ => None,
    }
}

fn validate_receipt(receipt: &Receipt, candidate_sha: &str) -> Result<(), String> {
    if receipt.source_revision != candidate_sha {
        return Err("receipt source revision does not match the expected candidate SHA".to_owned());
    }
    if receipt.source_revision_source != "explicit" || receipt.source_acceptance_status != "PASS" {
        return Err("attestable evidence requires explicit accepted source provenance".to_owned());
    }
    let expected_decoded = expected_decoded_bytes(&receipt.profile)
        .ok_or_else(|| format!("unexpected performance profile {}", receipt.profile))?;
    if receipt.decoded_bytes != expected_decoded || receipt.coded_bytes == 0 {
        return Err(format!("{} reported invalid payload sizes", receipt.profile));
    }
    if receipt.samples != SAMPLE_COUNT || receipt.budget_microseconds != P95_BUDGET_MICROSECONDS {
        return Err(format!("{} changed the governed sample/budget contract", receipt.profile));
    }
    if receipt.runtime_parallelism == 0 {
        return Err("runtime parallelism must be positive".to_owned());
    }
    if receipt.p50_microseconds > receipt.p95_microseconds
        || receipt.p95_microseconds > receipt.maximum_microseconds
    {
        return Err(format!("{} reported non-monotonic latency percentiles", receipt.profile));
    }
    let expected_budget_status = if receipt.p95_microseconds <= P95_BUDGET_MICROSECONDS {
        "PASS"
    } else {
        "FAIL"
    };
    if receipt.budget_status != expected_budget_status {
        return Err(format!("{} budget status disagrees with p95", receipt.profile));
    }
    Ok(())
}

fn parse_receipts(text: &str, candidate_sha: &str) -> Result<Vec<Receipt>, String> {
    let mut receipts = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_receipt_line)
        .collect::<Result<Vec<_>, _>>()?;
    if receipts.len() != 2 {
        return Err("stacked content-coding evidence requires exactly two receipt lines".to_owned());
    }
    for receipt in &receipts {
        validate_receipt(receipt, candidate_sha)?;
    }
    receipts.sort_by_key(|receipt| profile_rank(&receipt.profile).unwrap_or(usize::MAX));
    if profile_rank(&receipts[0].profile) != Some(0) || profile_rank(&receipts[1].profile) != Some(1) {
        return Err("stacked content-coding evidence requires both governed profiles".to_owned());
    }

    let first = &receipts[0];
    for receipt in &receipts[1..] {
        if receipt.environment_id != first.environment_id
            || receipt.runtime_os != first.runtime_os
            || receipt.runtime_arch != first.runtime_arch
            || receipt.runtime_parallelism != first.runtime_parallelism
            || receipt.network_authority != first.network_authority
            || receipt.network_acceptance_status != first.network_acceptance_status
            || receipt.evidence_authority != first.evidence_authority
            || receipt.evidence_acceptance_status != first.evidence_acceptance_status
        {
            return Err("performance receipts disagree on runtime or authority identity".to_owned());
        }
    }
    Ok(receipts)
}

fn result_document(candidate_sha: &str, receipts: &[Receipt]) -> String {
    let measurements = receipts
        .iter()
        .map(|receipt| {
            format!(
                "{{\"acceptance_status\":\"{}\",\"budget_status\":\"{}\",\"coded_bytes\":{},\"decoded_bytes\":{},\"maximum_microseconds\":{},\"p50_microseconds\":{},\"p95_microseconds\":{},\"profile\":\"{}\",\"samples\":{},\"source_acceptance_status\":\"{}\"}}",
                receipt.acceptance_status,
                receipt.budget_status,
                receipt.coded_bytes,
                receipt.decoded_bytes,
                receipt.maximum_microseconds,
                receipt.p50_microseconds,
                receipt.p95_microseconds,
                receipt.profile,
                receipt.samples,
                receipt.source_acceptance_status
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"budget_microseconds\":{P95_BUDGET_MICROSECONDS},\"candidate_sha\":\"{candidate_sha}\",\"measurements\":[{measurements}],\"schema_version\":\"1.0\",\"selected_profile\":\"{SELECTED_PROFILE}\"}}\n"
    )
}

fn runtime_document(candidate_sha: &str, receipt: &Receipt) -> String {
    format!(
        "{{\"candidate_sha\":\"{candidate_sha}\",\"environment_id\":\"{}\",\"evidence_acceptance_status\":\"{}\",\"evidence_authority\":\"{}\",\"network_acceptance_status\":\"{}\",\"network_authority\":\"{}\",\"runtime_arch\":\"{}\",\"runtime_os\":\"{}\",\"runtime_parallelism\":{},\"schema_version\":\"1.0\",\"selected_profile\":\"{SELECTED_PROFILE}\",\"source_revision_source\":\"{}\"}}\n",
        receipt.environment_id,
        receipt.evidence_acceptance_status,
        receipt.evidence_authority,
        receipt.network_acceptance_status,
        receipt.network_authority,
        receipt.runtime_arch,
        receipt.runtime_os,
        receipt.runtime_parallelism,
        receipt.source_revision_source
    )
}

fn fixture_document(candidate_sha: &str) -> String {
    format!(
        "{{\"candidate_sha\":\"{candidate_sha}\",\"fixture_kind\":\"deterministic_synthetic_no_external_dataset\",\"p95_budget_microseconds\":{P95_BUDGET_MICROSECONDS},\"profiles\":[{{\"content_encoding\":\"gzip, deflate\",\"decoded_bytes\":262144,\"name\":\"gzip-deflate-256k\"}},{{\"content_encoding\":\"deflate, gzip\",\"decoded_bytes\":1048576,\"name\":\"deflate-gzip-1m\"}}],\"sample_count\":{SAMPLE_COUNT},\"schema_version\":\"1.0\",\"seed_block_bytes\":65536,\"seed_initial_state\":\"9e3779b9\",\"selected_profile\":\"{SELECTED_PROFILE}\",\"transport\":\"authenticated_loopback_tls\"}}\n"
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn write_new_file(path: &Path, payload: &str) -> Result<String, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("create evidence file {}: {error}", path.display()))?;
    file.write_all(payload.as_bytes())
        .map_err(|error| format!("write evidence file {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync evidence file {}: {error}", path.display()))?;
    Ok(sha256_hex(payload.as_bytes()))
}

fn materialize_evidence(
    output_dir: &Path,
    result: &str,
    runtime: &str,
    fixture: &str,
) -> Result<EvidenceDigests, String> {
    fs::create_dir(output_dir).map_err(|error| {
        format!(
            "create fresh performance evidence directory {}: {error}",
            output_dir.display()
        )
    })?;
    let outcome = (|| {
        let result_sha256 = write_new_file(&output_dir.join(RESULT_FILENAME), result)?;
        let runtime_sha256 = write_new_file(&output_dir.join(RUNTIME_FILENAME), runtime)?;
        let fixture_sha256 = write_new_file(&output_dir.join(FIXTURE_FILENAME), fixture)?;
        Ok(EvidenceDigests {
            result_sha256,
            runtime_sha256,
            fixture_sha256,
        })
    })();
    if outcome.is_err() {
        let _ = fs::remove_dir_all(output_dir);
    }
    outcome
}

fn required_path_env(name: &str) -> Result<PathBuf, String> {
    let value = env::var(name).map_err(|error| format!("{name} must be set: {error:?}"))?;
    if value.is_empty() {
        return Err(format!("{name} must not be empty"));
    }
    Ok(PathBuf::from(value))
}

fn main() -> Result<(), String> {
    let candidate_sha = env::var(SOURCE_REVISION_ENV)
        .map_err(|error| format!("{SOURCE_REVISION_ENV} must be set explicitly: {error:?}"))?;
    let candidate_sha = parse_candidate_sha(&candidate_sha)?;
    let receipt_path = required_path_env(RECEIPT_PATH_ENV)?;
    let output_dir = required_path_env(EVIDENCE_DIR_ENV)?;
    let receipt_text = fs::read_to_string(&receipt_path)
        .map_err(|error| format!("read performance receipt {}: {error}", receipt_path.display()))?;
    let receipts = parse_receipts(&receipt_text, &candidate_sha)?;
    let result = result_document(&candidate_sha, &receipts);
    let runtime = runtime_document(&candidate_sha, &receipts[0]);
    let fixture = fixture_document(&candidate_sha);
    let digests = materialize_evidence(&output_dir, &result, &runtime, &fixture)?;

    println!(
        "performance_profile={SELECTED_PROFILE} result_filename={RESULT_FILENAME} result_sha256={} runtime_evidence_filename={RUNTIME_FILENAME} runtime_evidence_sha256={} fixture_filename={FIXTURE_FILENAME} fixture_sha256={}",
        digests.result_sha256, digests.runtime_sha256, digests.fixture_sha256
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        EVIDENCE_DIR_ENV, FIXTURE_FILENAME, P95_BUDGET_MICROSECONDS, RECEIPT_PATH_ENV,
        RESULT_FILENAME, RUNTIME_FILENAME, SAMPLE_COUNT, SELECTED_PROFILE, fixture_document,
        parse_candidate_sha, parse_receipts, result_document, runtime_document, sha256_hex,
    };

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

    fn receipt_line(profile: &str, decoded_bytes: usize, p95_us: u128) -> String {
        let budget_status = if p95_us <= P95_BUDGET_MICROSECONDS {
            "PASS"
        } else {
            "FAIL"
        };
        format!(
            "source_revision={SHA} source_revision_source=explicit environment_id=gh-ubuntu-24.04/x64@runner-4 runtime_os=linux runtime_arch=x86_64 runtime_parallelism=4 network_authority=parent_untimed_connection_plan network_acceptance_status=UNACCEPTED_PARENT_NETWORK_AUTHORITY evidence_authority=caller_produced_unattested_receipt evidence_acceptance_status=UNACCEPTED_UNATTESTED_RECEIPT profile={profile} decoded_bytes={decoded_bytes} coded_bytes=65536 samples={SAMPLE_COUNT} p50_us=1000 p95_us={p95_us} max_us=30000 budget_us={P95_BUDGET_MICROSECONDS} budget_status={budget_status} source_acceptance_status=PASS acceptance_status=UNACCEPTED_PARENT_NETWORK_AUTHORITY"
        )
    }

    fn receipts_text() -> String {
        format!(
            "{}\n{}\n",
            receipt_line("gzip-deflate-256k", 256 * 1024, 2_000),
            receipt_line("deflate-gzip-1m", 1024 * 1024, 25_000)
        )
    }

    #[test]
    fn candidate_sha_requires_exact_lowercase_git_identity() {
        assert_eq!(parse_candidate_sha(SHA).as_deref(), Ok(SHA));
        assert!(parse_candidate_sha("0123456789ABCDEF0123456789ABCDEF01234567").is_err());
        assert!(parse_candidate_sha("0123").is_err());
    }

    #[test]
    fn receipts_bind_both_profiles_to_one_runtime_and_candidate() -> Result<(), String> {
        let receipts = parse_receipts(&receipts_text(), SHA)?;
        assert_eq!(receipts.len(), 2);
        assert_eq!(receipts[0].profile, "gzip-deflate-256k");
        assert_eq!(receipts[1].profile, "deflate-gzip-1m");
        assert_eq!(receipts[0].budget_status, "PASS");
        assert_eq!(receipts[1].budget_status, "FAIL");
        Ok(())
    }

    #[test]
    fn evidence_documents_bind_candidate_profile_runtime_and_fixture() -> Result<(), String> {
        let receipts = parse_receipts(&receipts_text(), SHA)?;
        let result = result_document(SHA, &receipts);
        let runtime = runtime_document(SHA, &receipts[0]);
        let fixture = fixture_document(SHA);
        for document in [&result, &runtime] {
            assert!(document.contains(&format!("\"candidate_sha\":\"{SHA}\"")));
            assert!(document.contains(&format!(
                "\"selected_profile\":\"{SELECTED_PROFILE}\""
            )));
        }
        assert!(fixture.contains("deterministic_synthetic_no_external_dataset"));
        assert!(fixture.contains("authenticated_loopback_tls"));
        assert!(fixture.contains(&format!("\"candidate_sha\":\"{SHA}\"")));
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        Ok(())
    }

    #[test]
    fn central_intake_filenames_and_env_contract_are_stable() {
        assert_eq!(RESULT_FILENAME, "content-coding-result.json");
        assert_eq!(RUNTIME_FILENAME, "content-coding-runtime.json");
        assert_eq!(FIXTURE_FILENAME, "content-coding-fixture.json");
        assert_eq!(RECEIPT_PATH_ENV, "ORIGINWEAVE_PERFORMANCE_RECEIPT_PATH");
        assert_eq!(EVIDENCE_DIR_ENV, "ORIGINWEAVE_PERFORMANCE_EVIDENCE_DIR");
    }

    #[test]
    fn malformed_or_mixed_receipts_fail_closed() {
        let one = format!("{}\n", receipt_line("gzip-deflate-256k", 256 * 1024, 2_000));
        assert!(parse_receipts(&one, SHA).is_err());

        let wrong_sha = receipts_text().replace(SHA, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert!(parse_receipts(&wrong_sha, SHA).is_err());

        let fallback = receipts_text().replace(
            "source_revision_source=explicit",
            "source_revision_source=github_sha",
        );
        assert!(parse_receipts(&fallback, SHA).is_err());

        let mixed_runtime = receipts_text().replacen(
            "runtime_arch=x86_64",
            "runtime_arch=aarch64",
            1,
        );
        assert!(parse_receipts(&mixed_runtime, SHA).is_err());
    }
}
