# Agent harness

Every repository exposes the same commands; each tool reaches them its own
way.

## The contract

| Name | What it does |
| --- | --- |
| `check` | Compile, lint and test the project |
| `guards` | The project's checks over the changed files |
| `gate` | `check`, then `guards` |
| `install-hooks` | Wire the git pre-commit |

`gate`, `install-hooks`, `status` and `brief` are imported from
`just/harness.just` into `agent.just`; `check` and `guards` are the project's.
The command is `just agent gate`.

## Using the recipes

Import `just/harness.just` from `agent.just`:

```just
# agent.just
import ".agents/portable/just/harness.just"
```

It calls the project's `check` and `guards`; it defines no project command.

## The adapters

- A tool with lifecycle events gets a hook that runs `guards` and reports
  the failure: a Cline `PostToolUse` and `TaskComplete`, a Claude Code
  `PostToolUse` and `Stop`.
- A tool with only rules gets a pointer to the repository's instructions
  and the git hook.
- The git pre-commit is the floor: every tool gets it, and it needs no
  event API.

## Using the templates

- Cline: link `hooks/cline/PostToolUse` and `TaskComplete` into
  `.clinerules/hooks/`.
- Git: link `hooks/git/pre-commit` into `.githooks/`, then run
  `just install-hooks` once.
- Claude Code: its `settings.json` hooks call the same commands; keep them
  in the project, where the event names live.

## The rules

- **No check in the adapter**: an adapter translates an event and calls a
  project command; it never holds a check.
- **Fast hooks**: a hook runs `guards`, not `gate`; a tool that kills a
  hook after a timeout must not run the test suite.
- **Pointers, not copies**: a tool-specific rule file points to the
  repository's instructions.
- **Report once**: a failed check is fed back as context; a completion hook
  blocks completion while it fails.
