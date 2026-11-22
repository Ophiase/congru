#!/usr/bin/env bash
# Usage: on-stop.sh <check>    (Stop)
# Runs <check> through .agents/checks/run.sh over every file the working tree
# changed. It catches what the Edit|Write hook never saw, such as a file
# created through a Bash heredoc. Exit 2 makes Claude keep working, which fires this hook again,
# so the same unfixed report never blocks twice in a row.
set -uo pipefail

check=$1
root=${CLAUDE_PROJECT_DIR:-$(pwd)}
cd "$root" || exit 0
payload=$(cat)

session=$(printf '%s' "$payload" | jq -r '.session_id // "unknown"')
state="$(git rev-parse --absolute-git-dir)/claude-stop-$check.$session"

mapfile -t changed < <(.agents/portable/checks/changed-files.sh)
report=$(.agents/checks/run.sh "$check" "${changed[@]}")
if (($? != 1)); then
  rm -f "$state"
  exit 0
fi

# Only the file:line findings are hashed: a tool summary line may carry an
# elapsed time that changes on every run and would defeat the guard.
hash=$(printf '%s\n' "$report" | grep -E '^[^ ]+:[0-9]+' | sha256sum | cut -d' ' -f1)
[[ -f $state && $(cat "$state") == "$hash" ]] && exit 0
printf '%s' "$hash" >"$state"

printf '%s\n' "$report" >&2
exit 2
