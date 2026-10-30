# Project instructions

The rules are in [AGENTS.md](../AGENTS.md) at the repository root; read it
first and follow it over this pointer.

The gate is `just agent gate`. `.clinerules/hooks/PostToolUse` and
`TaskComplete` run `just guards`; `PreToolUse` blocks the commands the policy
denies. `just install-hooks` wires the same check for every commit.
