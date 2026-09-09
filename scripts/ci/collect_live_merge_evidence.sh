#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="${ORIGINWEAVE_REPOSITORY:-ContextualWisdomLab/OriginWeave}"
if [[ $# -gt 0 ]]; then
  EVIDENCE_DIR="$1"
  if [[ -e "$EVIDENCE_DIR" ]]; then
    if [[ ! -d "$EVIDENCE_DIR" ]]; then
      printf 'Evidence path is not a directory: %s\n' "$EVIDENCE_DIR" >&2
      exit 1
    fi
    if [[ -n "$(find "$EVIDENCE_DIR" -mindepth 1 -print -quit)" ]]; then
      printf 'Evidence directory must be empty: %s\n' "$EVIDENCE_DIR" >&2
      exit 1
    fi
  else
    mkdir -p "$EVIDENCE_DIR"
  fi
else
  EVIDENCE_DIR="$(mktemp -d /tmp/originweave-evidence.XXXXXX)"
fi
printf 'Evidence directory: %s\n' "$EVIDENCE_DIR" >&2

gh api --paginate --slurp "repos/$REPOSITORY/pulls?state=open&per_page=100" \
  > "$EVIDENCE_DIR/open-pr-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/open-pr-pages.json" \
  > "$EVIDENCE_DIR/open-prs.json"
jq '{
  open_pull_requests: length,
  non_draft: (map(select(.draft == false)) | length),
  draft: (map(select(.draft == true)) | length)
}' "$EVIDENCE_DIR/open-prs.json"

gh api --paginate --slurp "repos/$REPOSITORY/issues?state=open&per_page=100" \
  > "$EVIDENCE_DIR/open-issue-pages.json"
jq '[.[][]] | map(select(has("pull_request") | not)) | {
  open_non_pr_issues: length
}' "$EVIDENCE_DIR/open-issue-pages.json"

gh api "repos/$REPOSITORY/branches/main" \
  > "$EVIDENCE_DIR/main-branch.json"
gh api --paginate --slurp \
  "repos/$REPOSITORY/rules/branches/main?per_page=100" \
  > "$EVIDENCE_DIR/main-branch-rule-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/main-branch-rule-pages.json" \
  > "$EVIDENCE_DIR/main-branch-rules.json"
gh api --paginate --slurp \
  "repos/$REPOSITORY/collaborators?affiliation=all&per_page=100" \
  > "$EVIDENCE_DIR/collaborator-pages.json"
jq '[.[][]]' "$EVIDENCE_DIR/collaborator-pages.json" \
  > "$EVIDENCE_DIR/collaborators.json"

jq -r '.[].number' "$EVIDENCE_DIR/open-prs.json" | while read -r PR; do
  STABLE_HEAD=false
  INVENTORY_DRAFT=$(jq -r --argjson pr "$PR" \
    '.[] | select(.number == $pr) | .draft' "$EVIDENCE_DIR/open-prs.json")
  for ATTEMPT in 1 2 3; do
    VERDICT_PATH="$EVIDENCE_DIR/pr-${PR}-merge-verdict.json"
    VERDICT_TMP="$EVIDENCE_DIR/pr-${PR}-merge-verdict.json.tmp"
    PR_JSON="$EVIDENCE_DIR/pr-${PR}.json"
    RECHECKED_PR_JSON="$EVIDENCE_DIR/pr-${PR}-rechecked.json"
    rm -f "$VERDICT_PATH" "$VERDICT_TMP" "$RECHECKED_PR_JSON"

    gh api "repos/$REPOSITORY/pulls/$PR" > "$PR_JSON"
    HEAD_SHA=$(jq -r '.head.sha' "$PR_JSON")
    BASE_SHA=$(jq -r '.base.sha' "$PR_JSON")
    BASE_REF=$(jq -r '.base.ref' "$PR_JSON")
    PR_STATE=$(jq -r '.state' "$PR_JSON")
    PR_DRAFT=$(jq -r '.draft' "$PR_JSON")
    BASE_REF_ENCODED=$(jq -rn --arg value "$BASE_REF" '$value | @uri')

    gh api --paginate --slurp \
      "repos/$REPOSITORY/rules/branches/$BASE_REF_ENCODED?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-branch-rule-pages.json"
    jq '[.[][]]' "$EVIDENCE_DIR/pr-${PR}-branch-rule-pages.json" \
      > "$EVIDENCE_DIR/pr-${PR}-branch-rules.json"
    gh api --paginate --slurp \
      "repos/$REPOSITORY/commits/$HEAD_SHA/check-runs?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-check-runs.json"
    gh api --paginate --slurp \
      "repos/$REPOSITORY/commits/$HEAD_SHA/statuses?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-statuses.json"
    gh api --paginate --slurp \
      "repos/$REPOSITORY/pulls/$PR/reviews?per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-reviews.json"
    gh api --paginate --slurp \
      "repos/$REPOSITORY/actions/runs?head_sha=$HEAD_SHA&per_page=100" \
      > "$EVIDENCE_DIR/pr-${PR}-workflow-runs.json"
    gh api graphql --paginate --slurp \
      -F owner="${REPOSITORY%%/*}" \
      -F name="${REPOSITORY#*/}" \
      -F number="$PR" \
      -f query='
query($owner: String!, $name: String!, $number: Int!, $endCursor: String) {
  repository(owner: $owner, name: $name) {
    pullRequest(number: $number) {
      reviewThreads(first: 100, after: $endCursor) {
        nodes { id isResolved isOutdated }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
}' > "$EVIDENCE_DIR/pr-${PR}-review-threads.json"

    jq -n \
      --arg head "$HEAD_SHA" \
      --arg base "$BASE_SHA" \
      --slurpfile pr "$PR_JSON" \
      --slurpfile checks "$EVIDENCE_DIR/pr-${PR}-check-runs.json" \
      --slurpfile statuses "$EVIDENCE_DIR/pr-${PR}-statuses.json" \
      --slurpfile reviews "$EVIDENCE_DIR/pr-${PR}-reviews.json" \
      --slurpfile workflow_runs "$EVIDENCE_DIR/pr-${PR}-workflow-runs.json" \
      --slurpfile rules "$EVIDENCE_DIR/pr-${PR}-branch-rules.json" \
      --slurpfile collaborators "$EVIDENCE_DIR/collaborators.json" \
      --slurpfile threads "$EVIDENCE_DIR/pr-${PR}-review-threads.json" \
      '(
        [
          $rules[][]?
          | select(.type == "pull_request")
          | .parameters
        ] | first // {}
      ) as $pull_request_parameters
      | (
          [
            $reviews[][][]?
            | {reviewer: .user.login, state, submitted_at, commit_id}
            | select(.submitted_at != null)
            | select(.state == "APPROVED" or .state == "CHANGES_REQUESTED")
            | select(.reviewer != $pr[0].user.login)
            | select(.reviewer as $reviewer |
                any($collaborators[][]?;
                  .login == $reviewer and
                  (.permissions.push == true or
                   .permissions.maintain == true or
                   .permissions.admin == true)))
          ]
          | group_by(.reviewer)
          | map(sort_by(.submitted_at) | last)
        ) as $current_review_decisions
      | (
          $current_review_decisions
          | map(select(.state == "APPROVED" and .commit_id == $head))
        ) as $current_approvals
      | (
          $current_review_decisions
          | map(select(.state == "CHANGES_REQUESTED"))
        ) as $current_change_requests
      | (
          [$workflow_runs[][].workflow_runs[]?]
        ) as $all_workflow_runs
      | (
          $all_workflow_runs
          | map(select(
              any(.pull_requests[]?;
                .number == ($pr[0].number) and
                .head.sha == $head and
                .base.sha == $base)
            ))
        ) as $exact_pr_base_workflow_runs
      | ($pull_request_parameters.required_approving_review_count // 0) as $required_review_count
      | ($pull_request_parameters.require_last_push_approval // false) as $require_last_push_approval
      | {
          head_sha: $head,
          base_sha: $base,
          required_status_checks: {
            check_runs: [$checks[][].check_runs[]?],
            legacy_statuses: [$statuses[][][]?]
          },
          workflow_runs: $exact_pr_base_workflow_runs,
          workflow_runs_without_exact_pr_base_provenance: (
            ($all_workflow_runs | length) - ($exact_pr_base_workflow_runs | length)
          ),
          counted_approvals: ($current_approvals | length),
          blocking_change_requests: $current_change_requests,
          required_approving_review_count: $required_review_count,
          require_last_push_approval: $require_last_push_approval,
          last_push_approval_authority: (
            if $require_last_push_approval == true
            then "github_rule_evaluation_required"
            else "not_required"
            end
          ),
          approval_gate_satisfied: (
            if $require_last_push_approval == true then false
            else (
              (($current_approvals | length) >= $required_review_count) and
              (($current_change_requests | length) == 0)
            )
            end
          ),
          required_workflows: [
            $rules[][]?
            | select(.type == "workflows")
            | .parameters.workflows[]
          ],
          unresolved_threads: [
            $threads[][].data.repository.pullRequest.reviewThreads.nodes[]?
            | select(.isResolved == false)
          ]
        }' > "$VERDICT_TMP"

    RECHECKED_HEAD_SHA=$(gh api "repos/$REPOSITORY/pulls/$PR" \
      | tee "$RECHECKED_PR_JSON" \
      | jq -r '.head.sha')
    RECHECKED_BASE_SHA=$(jq -r '.base.sha' "$RECHECKED_PR_JSON")
    RECHECKED_BASE_REF=$(jq -r '.base.ref' "$RECHECKED_PR_JSON")
    RECHECKED_STATE=$(jq -r '.state' "$RECHECKED_PR_JSON")
    RECHECKED_DRAFT=$(jq -r '.draft' "$RECHECKED_PR_JSON")
    if [[ "$RECHECKED_HEAD_SHA" == "$HEAD_SHA" && \
          "$RECHECKED_BASE_SHA" == "$BASE_SHA" && \
          "$RECHECKED_BASE_REF" == "$BASE_REF" && \
          "$PR_STATE" == "open" && \
          "$RECHECKED_STATE" == "$PR_STATE" && \
          "$RECHECKED_DRAFT" == "$PR_DRAFT" && \
          "$PR_DRAFT" == "$INVENTORY_DRAFT" ]]; then
      mv "$VERDICT_TMP" "$VERDICT_PATH"
      mv "$RECHECKED_PR_JSON" "$PR_JSON"
      STABLE_HEAD=true
      break
    fi

    rm -f "$VERDICT_TMP" "$RECHECKED_PR_JSON"
    printf 'Discarding moving PR evidence for #%s (head %s -> %s, base %s/%s -> %s/%s, state %s -> %s, draft %s -> %s, inventory draft %s) and retrying.\n' \
      "$PR" "$HEAD_SHA" "$RECHECKED_HEAD_SHA" "$BASE_REF" "$BASE_SHA" "$RECHECKED_BASE_REF" "$RECHECKED_BASE_SHA" \
      "$PR_STATE" "$RECHECKED_STATE" "$PR_DRAFT" "$RECHECKED_DRAFT" "$INVENTORY_DRAFT" >&2
  done

  if [[ "$STABLE_HEAD" != true ]]; then
    rm -f "$EVIDENCE_DIR"/pr-${PR}-*.json
    printf 'Unable to collect stable exact PR evidence for #%s after 3 attempts.\n' "$PR" >&2
    exit 1
  fi
done
