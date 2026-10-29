#!/usr/bin/env bash
# Usage: on-edit.sh <check>    (PostToolUse on Edit|Write)
# Runs <check> through .agents/checks/run.sh on the file just written; exit 2
# hands its report back to Claude. The report is hashed per session and per
# file, so a finding the developer decided to keep does not block twice.
set -uo pipefail

check=$1
root=${CLAUDE_PROJECT_DIR:-$(pwd)}
payload=$(cat)
file=$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty')
[[ -n $file ]] || exit 0
[[ $(realpath -m "$file") == "$(realpath "$root")"/* ]] || exit 0

report=$("$root/.agents/checks/run.sh" "$check" "$file")
(($? == 1)) || exit 0

session=$(printf '%s' "$payload" | jq -r '.session_id // "unknown"')
file_key=$(printf '%s' "$file" | sha256sum | cut -c1-16)
state="$(git -C "$root" rev-parse --absolute-git-dir)/claude-$check.$session.$file_key"
hash=$(printf '%s' "$report" | sha256sum | cut -d' ' -f1)
[[ -f $state && $(cat "$state") == "$hash" ]] && exit 0
printf '%s' "$hash" >"$state"

printf '%s\n' "$report" >&2
exit 2
