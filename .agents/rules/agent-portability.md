# Agent portability

The checks and the recipes are the contract; an agent's hooks are an adapter
over them.

- **One source**: `just agent gate`, `just check`, `just guards` and
  `.agents/checks/run.sh` run the same checks for every tool. A finding only
  a hook can see is not a finding.
- **No check in the adapter**: a tool-specific folder names the check to run
  and translates its payload; it holds no rule of its own.
- **Pointers, not copies**: a tool that reads `.clinerules/` or
  `.kilocode/rules/` gets one paragraph pointing to `AGENTS.md`; a copied
  rule drifts from the original.
- **The floor is the git hook**: `just install-hooks` wires `.githooks/`;
  it works for every tool, including those with no lifecycle hook.
- **A new tool** is adopted when it can run `just agent gate` and follow the
  procedures of the two skills; nothing else is required.
