# Night report

Newest first. Every number is one sensor seed; the frames of a path are not
independent.

## 2026-11-06, 0.1.24

**Done**: the crate regression understood and fixed; a numeric bug in the
confirmation found; the answer and the follower rewritten; all of it moved into
`congru-track`. The contour term stays out.

Demo B, frames within 2° of 90, on three paths (0.75 chosen on; 0.55, 1.3 apart):

| Condition | Before the hold, 0.1.20 | 0.1.23 | Now |
| --- | --- | --- | --- |
| Room | 0, 0, 0 | 81, 77, 82 | **90, 83, 83** |
| All in a room | 0, 0, 0 | 68, 71, 54 | **75, 82, 60** |
| All effects | 84, 89, 76 | 79, 81, 69 | **85, 82, 81** |
| Shake | 90, 90, 88 | 88, 88, 88 | **90, 90, 90** |
| Sensor, lens, flex | 90, 90, 87 | 90, 90, 90 | 90, 90, 90 |

Scenes, within 2 % of the size, of 72:

| Scene | 0.1.23 | Now |
| --- | --- | --- |
| Crate among crates | 0 | **72** |
| Shelf of mugs | 45 | **72** |
| Gear from above | 47 | **72** |
| Street corner | 57 | **72** |
| Car close by | 39 | **48** |
| Car on a road | 42 | **54** |
| Gear on a bench | 63 | **72** |
| Bracket at a low angle | 68 | **72** |
| Bracket behind a crate | 60 | **66** |
| Bin of parts | 71 | **72** |
| Twin brackets | 71 | **72** |
| Bracket in a bin, mug close up, part on a table | 72 | 72 |

**What found it**, in order of gain:

| Finding | Effect |
| --- | --- |
| Answers came from the tracker's filter, lagging behind each jolt | Shake and all effects |
| Confirmation of a flat view read rounding, $10^{150}$ | The flat scenes |
| A short ICP after the rays gives the precision they lack | The rooms, the cars |
| The place's gains: a lag costing 1 to 2 % of the size | Mugs, gears |
| Most of the object must lie in the box | Crate: a twin won |
| A search still asks 0.3 of agreement | Worst seed 44 to 73, and the box can go |
| The tracker ran behind the rays for nothing | Three times faster in a room |

**Headline**: with no box at all, nothing from the truth, all in a room gives 74,
71, 69 of 90 on three seeds, and 65 at 1.3 turns.

Other seeds, with the box, all in a room: 75, 78, 73, 78 at 0.75 turns; 60, 63,
65, 63 at 1.3.

**Decided alone, for the author to check**:

- **The short ICP**: every point of the box stays, 1 cm gate on pairs, judged on
  all rays. I read it as a gate, not the cut the author refused: its start is
  the rays' pose, not the truth. [The
  argument](../ideas/method/what-answers.md).
- **Parametric forecast**: written, tested, not yet used by any scene.
- **Contour term left out**: room +3, but in a room −7, shelf −6.

**What fails**:

- **All in a room at 1.3 turns**: 60; a knock believes a flat wrong pose for ten
  frames. An amount bar on the search changed nothing: not kept.
- **All effects at 0.55 turns**: 82, under 89 before the hold.
- **The first frames**: nothing while the can hides the part; the nearest car misses
  a third of its run, 48 of 72.
- **Rotation on symmetric objects** says nothing: gear on a bench fell from 19
  to 0 within 2° while its surface error halved. A tooth off is right by ADD-S.
- **Fragile gains**: 0.7 and 0.3 on the turn fell to 11 of 90 on one path; a sampling
  change of 4 % moved the room from 86 to 90. A few frames are not a result.
- **The benchmark was not rerun**: its code paths are unchanged, its filter gains
  the same.
- **A search costs seconds**: following is near real time, finding is not.
