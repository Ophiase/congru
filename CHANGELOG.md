# CHANGELOG

Milestones of the whole project; the version is in `VERSION`, and every
crate of the workspace carries the same one.

## Unreleased

## 0.1.22 — 2026-11-05

- **The realistic scenes run the new pipeline**: four of six hold, from 0 of 72.
  Within 2 % of their size: bin of parts 72, part on a table 71, gear on a bench
  65, shelf of mugs 43 ([scenes](docs/benchmark/scenes.md)).
- **A surface error beside the rotation one**: a gear turned by one tooth is right.
- **The cars stay at 0**: the simulated sensor has 1 m of noise at 14 m and
  2.6 m at 23 m, for a car 1.5 m high.

## 0.1.21 — 2026-11-05

- **Holding on when a frame says nothing**: hidden goes on along the trajectory,
  refuted searches every orientation where it stood. All in a room reaches 68, 74
  and 57 of 90 on three paths, the room 78, 73 and 79
  ([the step](docs/hold-step.md)).
- **All effects recedes** by 7 or 8 frames, shake by 1 or 2; the realistic scenes
  stay at 0 of 72.
- **The simple situations have their own page**, apart from the realistic scenes.

## 0.1.20 — 2026-11-05

- **A pose is worth what its hits say of it**: the information of the hits, less
  the rays through, ranks the truth first ([why](ideas/method/pose-information.md)).
- **Demo B with a noisy box**: the room goes from 0 to 71 of 90, 77 and 69 on paths
  never used; with every effect, 41, then only 20 and 13
  ([the box](docs/box-step.md), [the step before](docs/shape-crop-step.md)).
- **The ladder is answered**: along rays, and read as segments without any start.
- **All effects recedes**, 84 to 79 of 90; the realistic scenes stay at 0 of 72.

## 0.1.19 — 2026-11-04

- **Detection judges every vote**, refined along rays and weighted by the share
  of the model in the image: all effects 84 / 90, shake 90 / 90. A room stays at
  0 / 90.
- **The failure is located**: votes favour planes, and the tracker undoes a right
  start while still trusting it ([evidence](ideas/method/ray-evidence.md)).
- **Visibility and a ladder of clouds**: what a ray says, and eight 2D steps, each
  measured against a naive fit ([ladder](ideas/problem/cloud-ladder.md)).

## 0.1.18 — 2026-11-04

- **Demo B made rougher and legible.** A cheap sensor as rough as demo A's,
  one animation per effect, a real fractal background, and a room that visibly
  still breaks the search.
- **Long jobs** have a rule; the Python skin moves to 3.14.
- **Kanon resynced.** The canon, the portable layer and the markdown check;
  every diagram follows the rule; reference projects carry a recorded consent.

## 0.1.17 — 2026-10-30

- **Agent portability.** The root justfile has a module `agent`; `agent.just`
  imports the portable `just/harness.just`, and `just agent gate` is the one
  command to declare work done.
- **Hooks and permissions.** Cline `.clinerules/hooks/`, Cursor
  `.cursor/hooks.json`, the git pre-commit and the Kilo `.kilo/kilo.jsonc`
  permissions call the same checks; the portable templates are symlinks.
- **Sharing standard.** `.agents/guard.yaml` and the local overrides; the
  Kanon and its `<>` sign.

## 0.1.16 — 2026-10-30

- **Shortcuts removed.** The demos no longer take their first pose from
  the truth, nor remove a supporting plane. The 0.1.15 entry claimed
  `CONGRU_SCENE` while it did not exist; it does now.
- **`Finder`**: a known object searched in a depth frame by point pair
  features, each candidate checked against the depth inside and just
  outside its silhouette. Tested on a clean frame. The 2D box it searches
  in is simulated from the truth, and said so.
- **[caution] The realistic scenes fail**: 0 of 72 frames within 2° on
  every scene; the turntable demo's table is to measure again.

## 0.1.15 — 2026-10-30

- **Lint fixed** in the breakdown drawing; `CONGRU_SCENE` picks the scene.

## 0.1.14 — 2026-10-30

- **One frame, step by step**: an image of every layer (depth, points,
  support removed, region, voxels, aligned) at the top of the pipeline page
  and of the scene gallery.

## 0.1.13 — 2026-10-30

- **CI runs only when started by hand**, as a rule in `AGENTS.md`: nothing
  runs or is paid for on a push.

## 0.1.12 — 2026-10-30

- **In plain words** at the top of the README, for anyone.
- **Beyond** (`ideas/beyond.md`): certificates, a commons of hard cases,
  cheap enough for everyone, understood not only used.

## 0.1.11 — 2026-10-30

- **The pipeline, layer by layer** (`docs/pipeline.md`): from a depth frame
  to a pose, what each layer does, where it lives, what milestone 2 adds.

## 0.1.10 — 2026-10-30

- **The scene tour starts from a detector's guess**, then from its last
  pose: the part on the table holds 72 of 72 frames, the gear 47 of 72
  (none before). Four scenes still end near 150° from a 6° guess, a
  convention bug to find.

## 0.1.9 — 2026-10-30

- **The scene tour in the gallery**, 4.5 MB: the part on the table is
  held; elsewhere the global search turns the target over, the next thing
  to fix.

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
