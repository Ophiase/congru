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
    cargo run --release -q -p congru-figures -- docs/figures

[doc("Run both demos: writes docs/figures/clouds.gif and tracking.gif, prints their tables")]
demo:
    cargo run --release -q -p congru-demo -- clouds docs/figures/clouds.gif
    cargo run --release -q -p congru-demo -- camera docs/figures/tracking.gif
    cargo run --release -q -p congru-demo -- tour docs/figures/scenes.gif
    cargo run --release -q -p congru-demo -- breakdown docs/figures/breakdown.gif

[doc("The published benchmark, on the evaluation seeds: resumes where it stopped, rewrites docs/benchmark/results.md as cells finish")]
bench:
    cargo run --release -q -p congru-bench -- docs/benchmark/results.md

[doc("Draw every benchmark situation as the methods receive it: docs/benchmark/scenes.md")]
bench-scenes:
    cargo run --release -q -p congru-bench -- --scenes

[doc("The published benchmark from scratch, after a change of the code")]
bench-fresh:
    cargo run --release -q -p congru-bench -- --fresh docs/benchmark/results.md

[doc("The benchmark on the tuning seeds, for trying ideas: writes target/bench-tuning.md")]
bench-tuning seeds="6":
    cargo run --release -q -p congru-bench -- --tuning --seeds {{ seeds }} target/bench-tuning.md

[doc("Build the C ABI and run the C++ example against it")]
capi-example:
    cargo build --release -q -p congru-capi
    g++ -std=c++20 -O2 -I crates/congru-capi/include crates/congru-capi/examples/align.cpp target/release/libcongru_capi.a -lpthread -ldl -lm -o target/release/congru-example
    target/release/congru-example

[doc("Everything that must pass before declaring work done")]
check: lint test rumdl

rumdl:
    rumdl check .

rumdl-fmt:
    rumdl fmt .

[doc("Build the Python bindings into crates/congru-python/.venv")]
[working-directory("crates/congru-python")]
python-dev:
    uv sync --no-install-project
    uv run --no-sync maturin develop --uv

[doc("Lint, format-check and test the Python bindings, rebuilt first")]
[working-directory("crates/congru-python")]
python-test: python-dev
    uv run --no-sync ruff check
    uv run --no-sync ruff format --check
    uv run --no-sync pytest

[doc("Run the project's and the portable checks over every changed file")]
[script("bash")]
guards:
    status=0
    for check in markdown french comment-slop doc-slop hardcoded-url portable rust-format folder-size yaml-extension config-format file-length version; do
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
