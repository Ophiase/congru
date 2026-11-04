# Reference projects — how the author works

The Kanon is the central source of the practices; the projects below are where
to look when a rule needs its example. Their paths and the author's `consent`
are in `.agents/local/projects.yaml` (gitignored; template:
[local.example.yaml](local.example.yaml)). Read only what is granted; take the
**method**, never the content.

**Absolute rule**: these projects are private. Never name them, quote them
or describe their content in a tracked file or a commit. Refer to them by
role only.

## Sibling reference

A Rust monorepo of the author, the most recent state of the conventions:
`ideas/` as the core, `.agents/portable/` copied from it, workspace lints,
`justfile` modules, hooks, the merge-master skill.

## Research reference

- `ideas/` first: short files, one notion each, a map, status tags.
- A report in **Typst**, figures drawn natively from data.
- Python to check by brute force, never to explain.

## Industrial Python reference

- `AGENTS.md` for every agent, `.claude/CLAUDE.md` importing it.
- Written architecture decisions, `justfile`, strict tooling (uv, ruff),
  changelog and version. The Python skin follows it.

## Founding case

A private report where the need first appeared. It may hold confidential
content: read locally only, take the need, never the content, and never
link this project to it in a tracked file.

## Before reusing

Each project's `.agents/guard.yaml`, then `.agents/local/guard.yaml`, says
what may be reused, attributed or published; read them first. A missing
guard is permissive, never confidential.
