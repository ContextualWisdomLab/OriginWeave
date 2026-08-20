#!/usr/bin/env python3
"""Classify exported GitHub Actions workflow records without mutating GitHub.

The utility consumes an operator-collected JSON document. It deliberately performs no
network request and has no workflow-disable capability. Its output binds every record
to one exact protected-branch revision, observation time, and complete pagination
receipt so a later authorized operator can review immutable workflow IDs safely.
"""

from __future__ import annotations

import argparse
import datetime
import json
import pathlib
import re
import sys
from typing import Any

_SCHEMA_VERSION = 1
_MAX_INPUT_BYTES = 4 * 1024 * 1024
_MAX_JSON_INTEGER_DIGITS = 20
_MAX_WORKFLOW_ID = (1 << 64) - 1
_MAX_RETRY_AFTER_SECONDS = 3600
_SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")
_TIMESTAMP_PATTERN = re.compile(
    r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"
)
_REPOSITORY_WORKFLOW_PREFIX = ".github/workflows/"
_DYNAMIC_WORKFLOW_PREFIX = "dynamic/"
_DISABLED_STATES = {
    "deleted",
    "disabled_fork",
    "disabled_inactivity",
    "disabled_manually",
}
_ALLOWED_STATES = {"active", *_DISABLED_STATES}
_RETRYABLE_HTTP_STATUSES = {429, 500, 502, 503, 504}


class WorkflowAuditError(ValueError):
    """Report malformed, incomplete, stale, or ambiguous registry evidence."""


class WorkflowAuditHttpStatusError(WorkflowAuditError):
    """Report one collected non-200 page with bounded retry guidance.

    The auditor still fails closed for every non-200 response. The ``retryable``
    flag only tells the external collector whether recollecting the page can be a
    safe bounded response to a reviewed throttling or transient server condition.
    A collected 403 remains non-retryable unless the collector also retained one
    bounded ``Retry-After`` delay; the delay is guidance for recollection only and
    never converts failed registry evidence into success.
    """

    def __init__(
        self,
        page_number: int,
        status_code: int,
        retry_after_seconds: int | None = None,
    ) -> None:
        super().__init__(f"registry page {page_number} did not return HTTP 200")
        self.page_number = page_number
        self.status_code = status_code
        self.retry_after_seconds = retry_after_seconds
        self.retryable = status_code in _RETRYABLE_HTTP_STATUSES or (
            status_code == 403 and retry_after_seconds is not None
        )


def _require_mapping(value: Any, field_name: str) -> dict[str, Any]:
    """Return a mapping or fail with a stable field-specific diagnostic."""

    if not isinstance(value, dict):
        raise WorkflowAuditError(f"{field_name} must be an object")
    return value


def _require_exact_fields(
    value: Any, field_name: str, expected_fields: set[str]
) -> dict[str, Any]:
    """Return a mapping whose member names exactly match this schema boundary."""

    mapping = _require_mapping(value, field_name)
    if set(mapping).difference(expected_fields):
        raise WorkflowAuditError(f"{field_name} contains an unsupported field")
    return mapping


def _require_list(value: Any, field_name: str) -> list[Any]:
    """Return a list or fail with a stable field-specific diagnostic."""

    if not isinstance(value, list):
        raise WorkflowAuditError(f"{field_name} must be an array")
    return value


def _require_nonempty_string(value: Any, field_name: str, maximum: int) -> str:
    """Return one bounded nonempty string without leading or trailing whitespace."""

    if not isinstance(value, str):
        raise WorkflowAuditError(f"{field_name} must be a string")
    if not value or value != value.strip():
        raise WorkflowAuditError(f"{field_name} is invalid")
    try:
        encoded_length = len(value.encode("utf-8"))
    except UnicodeEncodeError:
        raise WorkflowAuditError(f"{field_name} is invalid") from None
    if encoded_length > maximum:
        raise WorkflowAuditError(f"{field_name} is invalid")
    if any(not character.isprintable() for character in value):
        raise WorkflowAuditError(f"{field_name} contains a control character")
    return value


def _validate_sha(value: Any, field_name: str) -> str:
    """Return one exact lowercase forty-character Git commit SHA."""

    text = _require_nonempty_string(value, field_name, 40)
    if _SHA_PATTERN.fullmatch(text) is None:
        raise WorkflowAuditError(f"{field_name} must be a lowercase commit SHA")
    if text == "0" * 40:
        raise WorkflowAuditError(f"{field_name} must be a nonzero commit SHA")
    return text


def _validate_observed_at(value: Any) -> str:
    """Return one second-precision UTC observation timestamp."""

    text = _require_nonempty_string(value, "observed_at", 20)
    if _TIMESTAMP_PATTERN.fullmatch(text) is None:
        raise WorkflowAuditError("observed_at must use YYYY-MM-DDTHH:MM:SSZ")
    try:
        datetime.datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError:
        raise WorkflowAuditError(
            "observed_at must be a valid UTC calendar timestamp"
        ) from None
    return text


def _validate_reported_total_count(value: Any) -> int:
    """Return the nonnegative total reported by the first GitHub API page."""

    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise WorkflowAuditError("reported_total_count must be a nonnegative integer")
    return value


def _validate_retry_after_seconds(value: Any) -> int:
    """Return one bounded Retry-After delay for a failed collection page."""

    if (
        isinstance(value, bool)
        or not isinstance(value, int)
        or value < 0
        or value > _MAX_RETRY_AFTER_SECONDS
    ):
        raise WorkflowAuditError(
            "retry_after_seconds must be an integer from 0 through 3600"
        )
    return value


def _validate_workflow_path(value: Any, field_name: str) -> str:
    """Return one unambiguous GitHub workflow registry path."""

    path = _require_nonempty_string(value, field_name, 512)
    if "\\" in path or "%" in path:
        raise WorkflowAuditError(f"{field_name} contains encoded or alternate separators")
    segments = path.split("/")
    if any(segment in {"", ".", ".."} for segment in segments):
        raise WorkflowAuditError(f"{field_name} contains an ambiguous path segment")
    if path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
        workflow_name = path[len(_REPOSITORY_WORKFLOW_PREFIX) :]
        if "/" in workflow_name or not workflow_name.endswith((".yml", ".yaml")):
            raise WorkflowAuditError(
                f"{field_name} must name one direct .yml or .yaml workflow file"
            )
        return path
    if path.startswith(_DYNAMIC_WORKFLOW_PREFIX):
        return path
    if path.casefold().startswith(_REPOSITORY_WORKFLOW_PREFIX.casefold()):
        raise WorkflowAuditError(f"{field_name} changes canonical workflow path case")
    return path


def _validated_path_set(value: Any, field_name: str) -> set[str]:
    """Return a duplicate-free set of exact repository workflow paths."""

    paths = _require_list(value, field_name)
    validated: set[str] = set()
    for index, raw_path in enumerate(paths):
        path = _validate_workflow_path(raw_path, f"{field_name}[{index}]")
        if not path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
            raise WorkflowAuditError(f"{field_name}[{index}] is not a repository workflow")
        if path in validated:
            raise WorkflowAuditError(f"{field_name} contains a duplicate path")
        validated.add(path)
    return validated


def _validated_active_pr_owners(
    value: Any, active_pr_paths: set[str]
) -> dict[str, dict[str, Any]]:
    """Bind every deferred path to one PR and two equal exact head observations."""

    ownership_error = "active PR workflow ownership must be bound to exact PR heads"
    if value is None:
        if active_pr_paths:
            raise WorkflowAuditError(ownership_error)
        return {}

    owners = _require_list(value, "active_pr_workflow_owners")
    validated: dict[str, dict[str, Any]] = {}
    for index, raw_owner in enumerate(owners):
        owner = _require_exact_fields(
            raw_owner,
            f"active_pr_workflow_owners[{index}]",
            {"path", "pull_request_number", "head_sha", "observed_head_sha"},
        )
        path = _validate_workflow_path(
            owner.get("path"), f"active_pr_workflow_owners[{index}].path"
        )
        if not path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
            raise WorkflowAuditError(ownership_error)
        pull_request_number = owner.get("pull_request_number")
        if (
            isinstance(pull_request_number, bool)
            or not isinstance(pull_request_number, int)
            or pull_request_number <= 0
        ):
            raise WorkflowAuditError(ownership_error)
        head_sha = _validate_sha(
            owner.get("head_sha"), f"active_pr_workflow_owners[{index}].head_sha"
        )
        observed_head_sha = _validate_sha(
            owner.get("observed_head_sha"),
            f"active_pr_workflow_owners[{index}].observed_head_sha",
        )
        if head_sha != observed_head_sha:
            raise WorkflowAuditError("active PR workflow head moved during collection")
        if path in validated:
            raise WorkflowAuditError(ownership_error)
        validated[path] = {
            "pull_request_number": pull_request_number,
            "head_sha": head_sha,
        }

    if set(validated) != active_pr_paths:
        raise WorkflowAuditError(ownership_error)
    return validated


def _validate_pages(value: Any) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    """Return complete registry records and immutable pagination receipts."""

    raw_pages = _require_list(value, "registry_pages")
    if not raw_pages:
        raise WorkflowAuditError("registry_pages must not be empty")

    workflows: list[dict[str, Any]] = []
    receipts: list[dict[str, Any]] = []
    for index, raw_page in enumerate(raw_pages):
        page = _require_exact_fields(
            raw_page,
            f"registry_pages[{index}]",
            {"page", "status_code", "retry_after_seconds", "has_next", "workflows"},
        )
        expected_page = index + 1
        page_number = page.get("page")
        if (
            isinstance(page_number, bool)
            or not isinstance(page_number, int)
            or page_number != expected_page
        ):
            raise WorkflowAuditError("registry_pages must be contiguous and start at page 1")
        retry_after_seconds = None
        if "retry_after_seconds" in page:
            retry_after_seconds = _validate_retry_after_seconds(
                page.get("retry_after_seconds")
            )
        status_code = page.get("status_code")
        if isinstance(status_code, bool) or not isinstance(status_code, int):
            raise WorkflowAuditError(
                f"registry page {expected_page} did not return HTTP 200"
            )
        if status_code < 100 or status_code > 599:
            raise WorkflowAuditError(
                "status_code must be an integer from 100 through 599"
            )
        if status_code == 200 and retry_after_seconds is not None:
            raise WorkflowAuditError(
                "retry_after_seconds is only valid for a failed registry page"
            )
        if status_code != 200:
            raise WorkflowAuditHttpStatusError(
                expected_page, status_code, retry_after_seconds
            )
        has_next = page.get("has_next")
        if not isinstance(has_next, bool):
            raise WorkflowAuditError(f"registry page {expected_page} lacks has_next")
        is_last = index == len(raw_pages) - 1
        if has_next == is_last:
            raise WorkflowAuditError("registry pagination is truncated or contradictory")
        page_workflows = _require_list(
            page.get("workflows"), f"registry_pages[{index}].workflows"
        )
        workflows.extend(
            _require_mapping(record, f"registry_pages[{index}].workflows[{record_index}]")
            for record_index, record in enumerate(page_workflows)
        )
        receipts.append(
            {
                "page": expected_page,
                "status_code": 200,
                "item_count": len(page_workflows),
                "has_next": has_next,
            }
        )
    return workflows, receipts


def _classify_workflow(
    path: str,
    state: str,
    protected_paths: set[str],
    active_pr_paths: set[str],
) -> str:
    """Classify one registry record using exact path ownership, never display name."""

    if path in protected_paths:
        if state == "active":
            return "present_repository_workflow"
        return "disabled_present_repository_workflow"
    if path in active_pr_paths:
        if state == "active":
            return "active_pr_owned_workflow"
        return "disabled_active_pr_owned_workflow"
    if path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
        if state == "active":
            return "active_orphan_repository_workflow"
        return "disabled_orphan_repository_workflow"
    if path.startswith(_DYNAMIC_WORKFLOW_PREFIX):
        return "github_dynamic_workflow"
    return "unresolved_workflow_record"


def _validate_workflow_record(
    raw_record: dict[str, Any],
    record_index: int,
    seen_ids: set[int],
    seen_paths: set[str],
    protected_paths: set[str],
    active_pr_paths: set[str],
    active_pr_owners: dict[str, dict[str, Any]],
    default_branch_sha: str,
    observed_at: str,
) -> dict[str, Any]:
    """Validate and classify one exported GitHub Actions workflow record."""

    raw_record = _require_exact_fields(
        raw_record,
        f"workflow record {record_index}",
        {"id", "name", "path", "state"},
    )
    workflow_id = raw_record.get("id")
    if isinstance(workflow_id, bool) or not isinstance(workflow_id, int):
        raise WorkflowAuditError(f"workflow record {record_index} has an invalid id")
    if workflow_id <= 0 or workflow_id > _MAX_WORKFLOW_ID:
        raise WorkflowAuditError(f"workflow record {record_index} has an invalid id")
    if workflow_id in seen_ids:
        raise WorkflowAuditError(f"workflow record {record_index} reuses an id")
    seen_ids.add(workflow_id)

    name = _require_nonempty_string(
        raw_record.get("name"), f"workflow record {workflow_id} name", 256
    )
    path = _validate_workflow_path(
        raw_record.get("path"), f"workflow record {workflow_id} path"
    )
    if path in seen_paths:
        raise WorkflowAuditError(f"workflow record {workflow_id} reuses a path")
    seen_paths.add(path)
    state = _require_nonempty_string(
        raw_record.get("state"), f"workflow record {workflow_id} state", 64
    )
    if state not in _ALLOWED_STATES:
        raise WorkflowAuditError(f"workflow record {workflow_id} has an unknown state")

    classification = _classify_workflow(
        path, state, protected_paths, active_pr_paths
    )
    record = {
        "workflow_id": workflow_id,
        "name": name,
        "path": path,
        "state": state,
        "classification": classification,
        "disable_candidate": classification == "active_orphan_repository_workflow",
        "default_branch_sha": default_branch_sha,
        "observed_at": observed_at,
    }
    if classification in {
        "active_pr_owned_workflow",
        "disabled_active_pr_owned_workflow",
    }:
        record["active_pr_owner"] = active_pr_owners[path]
    return record


def audit_workflow_registry(payload: dict[str, Any]) -> dict[str, Any]:
    """Return credential-free, read-only workflow lifecycle evidence.

    The expected and observed protected-branch SHAs must be identical. Registry pages
    must be exhaustive and contiguous. The function never calls GitHub or mutates a
    workflow; a later authorized operator must independently refetch every candidate.
    """

    document = _require_exact_fields(
        payload,
        "payload",
        {
            "schema_version",
            "expected_default_branch_sha",
            "observed_default_branch_sha",
            "observed_at",
            "reported_total_count",
            "protected_workflow_paths",
            "active_pr_workflow_paths",
            "active_pr_workflow_owners",
            "registry_pages",
        },
    )
    schema_version = document.get("schema_version")
    if (
        isinstance(schema_version, bool)
        or not isinstance(schema_version, int)
        or schema_version != _SCHEMA_VERSION
    ):
        raise WorkflowAuditError("unsupported schema_version")

    expected_sha = _validate_sha(
        document.get("expected_default_branch_sha"), "expected_default_branch_sha"
    )
    observed_sha = _validate_sha(
        document.get("observed_default_branch_sha"), "observed_default_branch_sha"
    )
    if expected_sha != observed_sha:
        raise WorkflowAuditError("protected default branch moved during collection")
    observed_at = _validate_observed_at(document.get("observed_at"))
    reported_total_count = _validate_reported_total_count(
        document.get("reported_total_count")
    )

    protected_paths = _validated_path_set(
        document.get("protected_workflow_paths"), "protected_workflow_paths"
    )
    active_pr_paths = _validated_path_set(
        document.get("active_pr_workflow_paths"), "active_pr_workflow_paths"
    )
    active_pr_owners = _validated_active_pr_owners(
        document.get("active_pr_workflow_owners"), active_pr_paths
    )
    overlap = protected_paths.intersection(active_pr_paths)
    if overlap:
        raise WorkflowAuditError("protected and active-PR path ownership overlaps")

    raw_workflows, receipts = _validate_pages(document.get("registry_pages"))
    if len(raw_workflows) != reported_total_count:
        raise WorkflowAuditError(
            "reported_total_count does not match paginated workflow records"
        )
    seen_ids: set[int] = set()
    seen_paths: set[str] = set()
    records = [
        _validate_workflow_record(
            raw_record,
            record_index,
            seen_ids,
            seen_paths,
            protected_paths,
            active_pr_paths,
            active_pr_owners,
            observed_sha,
            observed_at,
        )
        for record_index, raw_record in enumerate(raw_workflows)
    ]
    records.sort(key=lambda record: record["workflow_id"])

    return {
        "schema_version": _SCHEMA_VERSION,
        "default_branch_sha": observed_sha,
        "observed_at": observed_at,
        "reported_total_count": reported_total_count,
        "mutation_performed": False,
        "pagination_receipts": receipts,
        "workflow_records": records,
    }


def _reject_duplicate_object_members(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    """Return one JSON object or fail closed when a member name is repeated."""

    result: dict[str, Any] = {}
    for name, value in pairs:
        if name in result:
            raise WorkflowAuditError("input contains a duplicate JSON object member")
        result[name] = value
    return result


def _reject_nonstandard_json_constant(value: str) -> Any:
    """Reject numeric constants that standards-conforming JSON does not define."""

    raise json.JSONDecodeError("non-standard JSON numeric constant", value, 0)


def _reject_json_floating_point(value: str) -> Any:
    """Reject floating-point numbers outside the schema-v1 evidence grammar."""

    raise json.JSONDecodeError("floating-point JSON numeric literal is unsupported", value, 0)


def _parse_bounded_json_integer(value: str) -> int:
    """Parse one canonical decimal JSON integer within the evidence digit budget."""

    if value == "-0":
        raise json.JSONDecodeError("negative-zero JSON integer is unsupported", value, 0)
    digits = value[1:] if value.startswith("-") else value
    if len(digits) > _MAX_JSON_INTEGER_DIGITS:
        raise json.JSONDecodeError("JSON integer literal exceeds digit bound", value, 0)
    return int(value)


def _read_payload(path: pathlib.Path) -> dict[str, Any]:
    """Read at most four mebibytes of unambiguous UTF-8 JSON audit evidence."""

    try:
        with path.open("rb") as source:
            content = source.read(_MAX_INPUT_BYTES + 1)
    except OSError as error:
        raise WorkflowAuditError("input is not readable UTF-8 JSON") from error
    if len(content) > _MAX_INPUT_BYTES:
        raise WorkflowAuditError("input exceeds the four-mebibyte audit bound")
    try:
        parsed = json.loads(
            content.decode("utf-8"),
            object_pairs_hook=_reject_duplicate_object_members,
            parse_constant=_reject_nonstandard_json_constant,
            parse_float=_reject_json_floating_point,
            parse_int=_parse_bounded_json_integer,
        )
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise WorkflowAuditError("input is not readable UTF-8 JSON") from error
    return _require_mapping(parsed, "payload")


def main(argv: list[str] | None = None) -> int:
    """Audit an exported registry document and emit canonical JSON evidence."""

    parser = argparse.ArgumentParser(
        description="Classify exported GitHub Actions workflow identities read-only."
    )
    parser.add_argument("input", type=pathlib.Path, help="bounded registry export JSON")
    parser.add_argument(
        "--output",
        type=pathlib.Path,
        help="optional evidence output path; stdout is used when omitted",
    )
    arguments = parser.parse_args(argv)
    try:
        evidence = audit_workflow_registry(_read_payload(arguments.input))
    except (OSError, WorkflowAuditError) as error:
        print(f"workflow registry audit failed: {error}", file=sys.stderr)
        return 1

    serialized = json.dumps(evidence, indent=2, sort_keys=True) + "\n"
    if arguments.output is None:
        sys.stdout.write(serialized)
        return 0
    try:
        arguments.output.write_text(serialized, encoding="utf-8")
    except OSError as error:
        print(f"workflow registry audit failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())