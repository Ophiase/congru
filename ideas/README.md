# ideas/ — map

The core of the project: **one concept per file**, short, informal first.
Code, README and paper are written from here; a choice that is not here yet
is not made yet.

Tags: **[defined]** (fixed by the author), **[sketch]** (argued, unchecked),
**[guess]**, **[to check]** (claim about prior art), **[question]** (the
author decides), **[caution]** (known trap).

## Thesis

Aligning two views of the same rigid object is a solved problem in papers
and an unsolved one in practice: every team rewrites ICP, tunes it on its
own data, and cannot ship it to the robot. The project is **one small,
typed, fast registration brick**, written once in Rust and called from
Python, C++ or firmware, whose behaviour under noise, outliers and partial
views is **measured against a known truth**, not hoped for.

## The path

```text
  origin.md ── what the author needed, and the 2025 attempt
      │
  problem/ ─── what is observed, under which hypotheses, in which variants
      │
  method/ ──── which algorithm answers which variant, and what is open
      │
  architecture/ ── how one core reaches every language and device
      │
  demo/ ────── a simulated robot tracking a part, the proof of M1
      │
  open.md ──── the questions still open
```

## Files

- [vision.md](vision.md) · [origin.md](origin.md) ·
  [milestones.md](milestones.md) · [naming.md](naming.md) ·
  [open.md](open.md) · [prior-art.md](prior-art.md)
- **problem/**:
  [observation-model](problem/observation-model.md) ·
  [hypotheses](problem/hypotheses.md) ·
  [variants](problem/variants.md) ·
  [what-we-know](problem/what-we-know.md) ·
  [measuring-success](problem/measuring-success.md) ·
  [sequences](problem/sequences.md)
- **method/**:
  [closed-form](method/closed-form.md) ·
  [icp-family](method/icp-family.md) ·
  [robustness](method/robustness.md) ·
  [global-initialisation](method/global-initialisation.md) ·
  [aggregation](method/aggregation.md) ·
  [mesh-targets](method/mesh-targets.md) ·
  [observability](method/observability.md) ·
  [tracking](method/tracking.md)
- **architecture/**:
  [layers](architecture/layers.md) ·
  [types](architecture/types.md) ·
  [bindings](architecture/bindings.md) ·
  [embedded](architecture/embedded.md) ·
  [performance](architecture/performance.md) ·
  [benchmark](architecture/benchmark.md) ·
  [stack](architecture/stack.md) ·
  [extension-points](architecture/extension-points.md)
- **demo/**: [README](demo/README.md) — A: point clouds over time;
  B: a depth camera (scene, sensor, degradations, perception, tracking,
  video)
