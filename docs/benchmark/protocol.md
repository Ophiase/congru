# Benchmark protocol

What congru is tested on, how, and how a result is scored. The tables
themselves are in [results.md](results.md), written by `just bench`; the
code of [congru-bench](../../crates/congru-bench/src/main.rs) is the
executable form of this page.

## The contract under test

Every method is an [`Aligner`](../../ideas/architecture/contract.md): it
owns its model, receives one frame (the observed points and, optionally, a
guess of the pose) and returns a pose with a flag saying whether it trusts
it. One-shot methods and trackers are run through the same loop.

## Hypotheses

Each hypothesis makes a faster or simpler method possible. The benchmark
states which one a situation breaks, so a method's limits are read off the
tables rather than discovered in the field.

| Hypothesis | Situations that break it |
| --- | --- |
| **H1 — rigid**: the object does not deform | Tracking *bend* |
| **H2 — mostly the object**: inliers outnumber outliers (an upstream mask or a crop isolated it) | Outliers 100 % and 300 % |
| **H3 — mostly visible**: the sensor sees most of the surface | Half and quarter visible, tracking *occluder* |
| **H4 — near**: a guess within about 20° exists | Guess 30°, guess 60°, no guess, tracking *knocks* |
| **H5 — calibrated**: the sensor's geometry is exact | Tracking *lens* and *shake* |
| **H6 — dense enough**: a few hundred points or more | Sparse |
| **H7 — asymmetric**: one pose, and only one, fits the shape | Every situation on the gear |

The library is first made to work under all seven, then each is dropped in
turn; a method that needs one says so in its documentation.

## Models

| Model | Why |
| --- | --- |
| **bracket** | A machined part: flat faces that leave motions free, thin plates whose two faces are close, small features (a boss, a pin) that pin the pose |
| **blob** | A smooth closed surface, nearly round: few features, near-symmetries, the trap of a wrong local minimum |
| **patch** | An open curved surface seen from one side, as a depth camera sees a part |
| **gear** | Twelve teeth: the shape is the same after a twelfth of a turn, so several poses fit it equally well; a method should not claim a pose it cannot tell from another |

Every distance is a share of the model's size, the diagonal of its bounding
box, so one situation means the same thing on every model.

## The statistical model of one frame

The observation model of [ideas/problem/](../../ideas/problem/observation-model.md),
with every parameter explicit:

| Effect | Model | Parameter |
| --- | --- | --- |
| Sampling | The surface sampled evenly, one point per 1/40 of the size, then a share kept | Density |
| Noise | Isotropic Gaussian on every observed point | σ, as a share of the size |
| Outliers | Uniform in the bounding box grown by a tenth | Count, as a share of the observed points |
| Partial view | Only the points on one side of a random plane | Visible share |
| Motion | A rotation uniform over SO(3), a translation in a ball of one size | — |
| Guess | The truth turned about the model's centre on a random axis, and shifted in a random direction | Angle, shift |

## Realistic scenes

Each scene of [scenes/](../../scenes/) is rendered by a simulated depth
camera, degraded as by a consumer sensor (noise growing with depth, holes,
flying pixels), stripped of the largest plane wider than the target (the
table, the road, the shelf board), and cropped around a detector's guess,
ten degrees and three percent of the target's size off. The methods then
find the target in what is left: clutter, neighbours, partial views.

## Situations

One trouble at a time on top of a nominal frame, then all together.

| Situation | Noise σ | Outliers | Visible | Density | Guess |
| --- | --- | --- | --- | --- | --- |
| nominal | 0.5 % | 0 | 100 % | 100 % | 10°, 2 % |
| noise 2 % | 2 % | 0 | 100 % | 100 % | 10°, 2 % |
| noise 5 % | 5 % | 0 | 100 % | 100 % | 10°, 2 % |
| outliers 30 % | 0.5 % | 30 % | 100 % | 100 % | 10°, 2 % |
| outliers 100 % | 0.5 % | 100 % | 100 % | 100 % | 10°, 2 % |
| outliers 300 % | 0.5 % | 300 % | 100 % | 100 % | 10°, 2 % |
| half visible | 0.5 % | 0 | 50 % | 100 % | 10°, 2 % |
| quarter visible | 0.5 % | 0 | 25 % | 100 % | 10°, 2 % |
| sparse | 0.5 % | 0 | 100 % | 5 % | 10°, 2 % |
| guess 30° | 0.5 % | 0 | 100 % | 100 % | 30°, 5 % |
| guess 60° | 0.5 % | 0 | 100 % | 100 % | 60°, 10 % |
| no guess | 0.5 % | 0 | 100 % | 100 % | 180°, 30 % |
| everything | 2 % | 30 % | 50 % | 30 % | 20°, 5 % |

## Sequences

A model moves in front of a sensor for 80 frames: it turns half a turn and
sways, 2.5 sizes away. The baseline is already hard: 10 % of the visible
surface comes back, with σ = 2.5 % noise and 40 points of clutter per
frame. Each sequence adds one trouble:

| Sequence | Added |
| --- | --- |
| noise | nothing |
| shake | the sensor jolts every frame: 0.6° and 2 % of the size |
| bend | the object flexes, up to 2 % of the size, with no rigid part to first order |
| occluder | a ball of 30 % of the size crosses in front |
| lens | radial distortion, `k = 1` |
| knocks | two abrupt turns of about 70° and 60° at frames 30 and 60 |

Every method gets the same first guess, 10° off, as from a detector; a
one-shot method then starts each frame from its own last answer.

## Metrics

| Metric | Definition |
| --- | --- |
| **Success** | Rotation error below 2° and translation error below 2 % of the size |
| **Success rate** | Share of successful trials, with its 95 % Wilson interval |
| **Confidently wrong** | Share of trials where the method trusted a pose that is not a success: for a robot, the worst outcome |
| **Rotation error** | Angle of the rotation between estimate and truth; median and 90th percentile over all trials, failures included |
| **Time** | Median wall-clock time of one `align` call, model already built |

A trial is one frame: one seed of a situation, or one frame of a sequence.

## Seeds

Two disjoint pools, 16 seeds per situation and 2 per sequence by default.
**Tuning seeds** (from 0) are for trying ideas:
`just bench-tuning`. **Evaluation seeds** (from 1 000 000) write the
published tables: `just bench`. A parameter is never chosen by looking at
the evaluation tables; a method tuned on them would be graded on its
homework.

## Aggregation

The leaderboard gives every situation and model the same weight: the mean
success rate, the worst situation, the mean confidently-wrong rate, and the
median time. The recommendations pick, for each class of situation and each
model, the method with the highest mean success, ties broken by time. They
inform the library's defaults; no default cites its table yet.

## Adding a method

1. Implement [`Aligner`](../../crates/congru/src/aligner/mod.rs).
2. Add one line to the catalogue in
   [methods.rs](../../crates/congru-bench/src/catalog/methods.rs).
3. Run `just bench-tuning` while developing, then `just bench`: finished
   cells are kept in `target/bench/`, so only the new method is computed,
   and the page is rewritten as its cells finish. After a change to code
   that every method shares, `just bench-fresh`. Commit the new
   [results.md](results.md) with the method.

A method may be better in one class and worse in another: the tables are
there to say where, not to crown a single winner.
