"""Fail-closed read-only classification for GitHub Actions workflow registry records.

The GitHub Actions registry can retain workflow identities after the corresponding
repository file has been deleted. This module deliberately performs no network or
mutation operation; callers must supply a complete paginated registry snapshot and
the exact protected-branch workflow paths observed for the same audit decision.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from string import hexdigits
from typing import Iterable, Mapping, Sequence

_REPOSITORY_WORKFLOW_PREFIX = ".github/workflows/"


class WorkflowRegistryAuditError(ValueError):
    """Raised when registry evidence is incomplete, ambiguous, or malformed."""


@dataclass(frozen=True)
class WorkflowRegistryAuditRecord:
    """Credential-free evidence for one workflow identity in one complete audit."""

    workflow_id: int
    path: str
    state: str
    classification: str
    default_branch_sha: str
    observed_at: str
    page_number: int


def _validate_default_branch_sha(value: str) -> None:
    """Require one exact full Git commit SHA rather than a symbolic or short ref."""

    if len(value) != 40 or any(character not in hexdigits for character in value):
        raise WorkflowRegistryAuditError("invalid default branch sha")


def _validate_observation_time(value: str) -> None:
    """Require an explicit timezone-aware ISO-8601 observation timestamp."""

    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise WorkflowRegistryAuditError("invalid observation time") from error
    if parsed.tzinfo is None:
        raise WorkflowRegistryAuditError("invalid observation time")


def _validate_repository_path(path: str) -> None:
    """Reject encoded or case-ambiguous attempts to masquerade as repository paths."""

    if not path:
        raise WorkflowRegistryAuditError("invalid workflow path")
    if "%" in path:
        raise WorkflowRegistryAuditError("ambiguous workflow path")
    if path.casefold().startswith(_REPOSITORY_WORKFLOW_PREFIX.casefold()) and not path.startswith(
        _REPOSITORY_WORKFLOW_PREFIX
    ):
        raise WorkflowRegistryAuditError("ambiguous workflow path")


def _validated_present_paths(paths: Iterable[str]) -> frozenset[str]:
    """Return exact protected-branch workflow paths after fail-closed validation."""

    validated: set[str] = set()
    for path in paths:
        if not isinstance(path, str):
            raise WorkflowRegistryAuditError("invalid workflow path")
        _validate_repository_path(path)
        if not path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
            raise WorkflowRegistryAuditError("invalid protected workflow path")
        validated.add(path)
    return frozenset(validated)


def _validate_pages(pages: Sequence[Mapping[str, object]]) -> None:
    """Require a complete contiguous sequence of pagination receipts beginning at page one."""

    if not pages:
        raise WorkflowRegistryAuditError("missing workflow registry pagination")

    for index, page in enumerate(pages):
        expected_page_number = index + 1
        page_number = page.get("page_number")
        if page_number != expected_page_number:
            raise WorkflowRegistryAuditError("non-contiguous workflow registry pagination")

        next_page = page.get("next_page")
        if index < len(pages) - 1:
            if next_page != expected_page_number + 1:
                raise WorkflowRegistryAuditError(
                    "non-contiguous workflow registry pagination"
                )
        elif next_page is not None:
            raise WorkflowRegistryAuditError("incomplete workflow registry pagination")


def audit_workflow_registry_pages(
    *,
    pages: Sequence[Mapping[str, object]],
    present_workflow_paths: Iterable[str],
    default_branch_sha: str,
    observed_at: str,
) -> tuple[WorkflowRegistryAuditRecord, ...]:
    """Classify one complete read-only Actions registry snapshot.

    `pages` must be caller-supplied exhaustive pagination evidence. Repository workflow
    path comparison is exact and case-sensitive; encoded or case-ambiguous paths fail
    closed rather than being normalized into a mutation candidate. The function never
    calls GitHub and never enables or disables a workflow.
    """

    _validate_default_branch_sha(default_branch_sha)
    _validate_observation_time(observed_at)
    present_paths = _validated_present_paths(present_workflow_paths)
    _validate_pages(pages)

    seen_workflow_ids: set[int] = set()
    records: list[WorkflowRegistryAuditRecord] = []

    for page in pages:
        page_number = page["page_number"]
        workflows = page.get("workflows")
        if not isinstance(workflows, Sequence) or isinstance(workflows, (str, bytes)):
            raise WorkflowRegistryAuditError("invalid workflow registry page")

        for workflow in workflows:
            if not isinstance(workflow, Mapping):
                raise WorkflowRegistryAuditError("invalid workflow registry record")

            workflow_id = workflow.get("id")
            path = workflow.get("path")
            state = workflow.get("state")
            if (
                not isinstance(workflow_id, int)
                or isinstance(workflow_id, bool)
                or workflow_id <= 0
            ):
                raise WorkflowRegistryAuditError("invalid workflow id")
            if workflow_id in seen_workflow_ids:
                raise WorkflowRegistryAuditError("duplicate workflow id")
            seen_workflow_ids.add(workflow_id)

            if not isinstance(path, str):
                raise WorkflowRegistryAuditError("invalid workflow path")
            _validate_repository_path(path)
            if not isinstance(state, str) or not state:
                raise WorkflowRegistryAuditError("invalid workflow state")

            if state != "active":
                classification = "disabled"
            elif path.startswith(_REPOSITORY_WORKFLOW_PREFIX):
                classification = (
                    "active_present" if path in present_paths else "active_orphan"
                )
            else:
                classification = "non_repository_path"

            records.append(
                WorkflowRegistryAuditRecord(
                    workflow_id=workflow_id,
                    path=path,
                    state=state,
                    classification=classification,
                    default_branch_sha=default_branch_sha,
                    observed_at=observed_at,
                    page_number=page_number,
                )
            )

    return tuple(records)
