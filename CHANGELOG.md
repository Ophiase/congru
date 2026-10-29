# CHANGELOG

Milestones of the whole project; the version is in `VERSION`, and every
crate of the workspace carries the same one.

## Unreleased

## 0.1.0 — 2026-10-29

The restart: from a problem statement to a measured library.

- **Ideas first**: `ideas/` holds the observation model of 2025, its
  hypotheses and variants, the methods, the architecture, the demos and the
  hypothesis ladder.
- **The core, congru**: weighted closed form, ICP point-to-point and
  point-to-plane with robust kernels, trimming and a motion prior,
  multistart over SO(3), mesh targets, and an observability report of the
  motions a frame leaves free.
- **One contract, `Aligner`**, for every method, one-shot or tracking; four
  methods in the core (local, search, annealed, guarded) and a two-phase
  tracker that follows, then finds again.
- **A benchmark**: three models, thirteen registration situations, six
  tracking sequences, success rates with Wilson intervals, the share of
  confidently wrong answers, and recommendations per class of situation.
- **Two demos** as animations: noisy point clouds tracked through shake,
  bending, an occluder, lens distortion and knocks; a part on a turntable
  under a simulated depth camera.
- **Python bindings**, NumPy in and out.
- **Conventions**: `AGENTS.md`, portable rules and checks, hooks, a review
  skill, continuous integration.
