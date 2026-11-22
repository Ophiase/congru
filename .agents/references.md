# Reference projects

The projects an agent may learn from are in `.agents/local/projects.yaml`,
gitignored, with where each one is and why it is readable. Read only; take
the **method**, never the content.

**Absolute rule**: treat every one of them as private unless its
`.agents/guard.yaml` says otherwise. Never name them, quote them or describe
their content in a tracked file or a commit. Refer to them by role only, and
keep even the roles in `.agents/local/`.

## Transitive lookup

A new project does not inherit these references, and it does not read them by
default. Its agent discovers them through Kanon's
`.agents/local/projects.yaml`, then follows this protocol:

1. **Classify** the entries by their `why` against the new project's needs,
   and keep the plausible ones.
2. **Propose** a shortlist to the user: one role, one reason, and why each one
   matches. Do not open a project before this step.
3. **Ask** before reading, project by project. The user may grant one, veto
   another, or leave it undecided.
4. **Read only the granted ones**, and obey the project's `.agents/guard.yaml`
   before reusing anything.
5. **Record** the answers in the new project's own
   `.agents/local/projects.yaml`. Kanon's file is the catalog, not the
   per-project permission list.

The `consent` field carries the standing decision:

| Value | Meaning |
| --- | --- |
| absent or `ask` | not decided; ask before reading |
| `granted` | the user pre-authorized this project |
| `denied` | the user vetoed it; never read it |

A `denied` wins over any `why`: a project described as relevant may still be
off limits. The user's word is the only permission that counts.

## Before reusing

Each project's `.agents/guard.yaml`, then `.agents/local/guard.yaml`, says
what may be reused, attributed or published; read them first. A missing
guard is permissive, never confidential.
