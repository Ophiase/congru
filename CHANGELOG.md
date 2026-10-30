# CHANGELOG

Milestones of the whole project; the version is in `VERSION`, and every
crate of the workspace carries the same one.

## Unreleased

- **The scene tour's animation** renders in half a minute now, but the
  tracker turns most targets over and the file weighs 31 MB: kept out until
  both are fixed.

## 0.1.8 — 2026-10-30

- **C ABI and C++20 header**, unstable until 1.0: a reusable target, ICP,
  the closed form; `just capi-example` runs a C++ program against it.
- **A `congru` command** printing JSON: align, track a folder of frames,
  describe a file, list the methods; `.xyz` and `.ply`.
- **The scene tour 200 times faster**: the model's sampling follows its
  size, so a car is no longer sampled every 3 mm.

## 0.1.7 — 2026-10-30

- **Scene tour** (`congru-demo tour`): every realistic scene filmed by a
  wandering camera while its target moves, tracked from a detector's crop;
  `CONGRU_FRAMES` sets the length.
- **Faster support removal**: the plane is searched on 3,000 points at
  most.

## 0.1.6 — 2026-10-29

- **The animations in pastel**, on warm paper; demo B in a room whose wall
  and bench have fractal relief.

## 0.1.5 — 2026-10-29

- **Realistic scenes**: six ordinary scenes in short TOML files (a part on
  a table, a car on a road, a bin of parts, a shelf of mugs, a gear on a
  bench, a street corner), built from a parametric prop library and a
  fractal ground, rendered in the gallery.
- **A realistic suite** in the benchmark: support plane removed, a
  detector's crop, every one-shot method on every scene.
- **A pastel palette** for the animations, a fractal room for demo B.
- **The roadmap** to a reference library, in `ideas/milestones.md`.

## 0.1.4 — 2026-10-29

- **Scene sheets as small PNGs** on pale paper, a few hundred kilobytes each.
- **Dependencies**: actions/checkout 7, extractions/setup-just 4.

## 0.1.3 — 2026-10-29

- **Scenes as sheets**: one image per model, every situation in a grid.
- **results.csv** is the benchmark's source of truth; the page lays it out.
- **Every method a link**, to Wikipedia or to its paper; a rule for agents.

## 0.1.2 — 2026-10-29

- **A camera in a hand**: both demos' shake now wanders slowly and widely,
  octaves of smooth noise, up to 6° and 2.5 cm over two seconds, with the
  jolts on top; tracking holds 109 of 120 frames and 90 of 90 within 2°.
- **Demo B's tiles** name their condition and show their error.

## 0.1.1 — 2026-10-29

More methods, a resumable benchmark, a harder demo B.

- **Consensus**, a global method from the shape: FPFH descriptors, mutual
  matches, RANSAC over triples, a check on the whole shape, then ICP.
- **Adaptive ICP**: kernel and gate follow the noise measured at each stage.
- **The benchmark resumes** where it stopped and rewrites its tables as
  cells finish; every scene is drawn as the methods receive it.
- **Demo B on the shared tracker**, with camera shake, lens distortion, a
  flexing part and a whole room in view.
- **The method landscape** in the README: ICP is one family among several.
- **Still open**: the palette, demo B's background and outline, and real
  scenes; see `ideas/open.md`.

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
