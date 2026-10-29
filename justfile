default:
    @just --list

[doc("Formatting and pedantic clippy, warnings as errors")]
lint:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings

[doc("Run every test of the workspace")]
test:
    cargo test --workspace

[doc("Format the Rust code")]
fmt:
    cargo fmt

[doc("Build the API documentation and open it")]
doc:
    cargo doc --workspace --no-deps --open

[doc("Recompute the figures of the documentation from real runs, into docs/figures/")]
figures:
    cargo run --release -q -p recalage-bench -- docs/figures

[doc("Run the simulated tracking demo: writes docs/figures/tracking.gif and prints the errors")]
demo:
    cargo run --release -q -p recalage-demo -- docs/figures/tracking.gif

[doc("Everything that must pass before declaring work done")]
check: lint test rumdl

rumdl:
    rumdl check .

rumdl-fmt:
    rumdl fmt .

[doc("Run the project's and the portable checks over every changed file")]
[script("bash")]
guards:
    status=0
    for check in markdown french comment-slop doc-slop hardcoded-url portable rust-format folder-size yaml-extension config-format file-length; do
        just _guard "$check" || status=1
    done
    exit $status

_guard check:
    @.agents/portable/checks/changed-files.sh | xargs -r -d '\n' .agents/checks/run.sh {{ check }}

[doc("Audit licences, advisories and wildcard dependencies")]
deny:
    cargo deny check --config deny.toml

[doc("The longest files of code, tests left out: green to 100 lines, yellow to 200, red past")]
[script("bash")]
longest count="20":
    set -euo pipefail
    git ls-files --cached --others --exclude-standard -- '*.rs' | while read -r file; do
        awk '/^#\[cfg\(test\)\]/ { exit } { lines++ } END { print lines + 0 }' "$file" | sed "s|\$| $file|"
    done | sort -rn | head -n "{{ count }}" | while read -r lines file; do
        if ((lines > 200)); then colour=31; elif ((lines > 100)); then colour=33; else colour=32; fi
        printf '\e[%sm%5d\e[0m  %s\n' "$colour" "$lines" "$file"
    done
