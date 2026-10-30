#!/usr/bin/env bash
# Usage: version.sh <file>...
# Reports a VERSION that differs from the workspace version of Cargo.toml,
# or that has no section in CHANGELOG.md. Runs whatever files it is given.
set -uo pipefail

root=$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)
version=$(tr -d '[:space:]' <"$root/VERSION")
cargo=$(awk '/^\[workspace.package\]/ { inside = 1; next } /^\[/ { inside = 0 } inside && /^version/ { gsub(/[" ]/, "", $0); split($0, parts, "="); print parts[2]; exit }' "$root/Cargo.toml")

status=0
if [[ $version != "$cargo" ]]; then
  printf 'VERSION says %s, Cargo.toml says %s: bump both together.\n' "$version" "$cargo"
  status=1
fi
citation=$(awk '/^version:/ { print $2; exit }' "$root/CITATION.cff")
if [[ $version != "$citation" ]]; then
  printf 'VERSION says %s, CITATION.cff says %s: bump both together.\n' "$version" "$citation"
  status=1
fi
if ! grep -q "^## $version " "$root/CHANGELOG.md"; then
  printf 'CHANGELOG.md has no section for %s.\n' "$version"
  status=1
fi
exit $status
