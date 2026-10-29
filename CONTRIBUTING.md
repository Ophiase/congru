# Contributing

The project is young and has one maintainer. Issues that bring a real
registration case (data, constraints, what failed) are the most useful
contribution today.

## Set up

- [just](https://just.systems), stable Rust 1.89 or newer with clippy and
  rustfmt, and [rumdl](https://github.com/rvben/rumdl) for markdown.
- `just --list` shows the recipes.

## Rules that matter

- **Ideas before code**: a new concept goes into [ideas/](ideas/README.md),
  one short file with its status tag, before the code relying on it.
- `just check` passes, and `just guards` over the changed files.
- A folder of code is one concept, seven entries at most
  ([layout](.agents/rules/layout.md)).
- A comment says a why the code cannot
  ([comments](.agents/portable/rules/comments.md)).
- A test checks a property against the synthetic truth, never a number
  copied from a run.

The full set is in [AGENTS.md](AGENTS.md), for people and agents alike.

## Commits

An imperative English summary of what changes; the body says why.
