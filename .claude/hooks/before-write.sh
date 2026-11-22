#!/usr/bin/env bash
# Usage: before-write.sh <check>    (PreToolUse on Write)
# Runs the gate <check> through .agents/checks/run.sh on the file about to be
# written, and denies the write with the gate's reason when it refuses.
set -uo pipefail

check=$1
root=${CLAUDE_PROJECT_DIR:-$(pwd)}
file=$(jq -r '.tool_input.file_path // empty')
[[ -n $file ]] || exit 0

reason=$("$root/.agents/checks/run.sh" "$check" "$file")
(($? == 1)) || exit 0

jq -n --arg reason "$reason" '{
  hookSpecificOutput: {
    hookEventName: "PreToolUse",
    permissionDecision: "deny",
    permissionDecisionReason: $reason
  }
}'
