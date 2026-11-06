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
| All in a room | 0, 0, 0 | 68, 71, 54 | **78, 81, 61** |
| All effects | 84, 89, 76 | 79, 81, 69 | **87, 84, 84** |
| Shake | 90, 90, 88 | 88, 88, 88 | **90, 90, 90** |
| Sensor, lens, flex | 90, 90, 87 | 90, 90, 90 | 90, 90, 90 |

Scenes, within 2 % of the size, of 72:

| Scene | 0.1.23 | Now |
| --- | --- | --- |
| Crate among crates | 0 | **72** |
| Shelf of mugs | 45 | **72** |
| Gear from above | 47 | **72** |
| Street corner | 57 | **72** |
| Car close by | 39 | **60** |
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
| The tracker ran behind the rays for nothing | Three times faster in a room |

**Decided alone, for the author to check**:

- **The short ICP**: every point of the box stays, 1 cm gate on pairs, judged on
  all rays. I read it as a gate, not the cut the author refused: its start is
  the rays' pose, not the truth. [The
  argument](../ideas/method/what-answers.md).
- **A lower bar for a search in a box**, 0.1 against 0.3: first frames now answer
  2 to 4° off instead of nothing.
- **Parametric forecast**: written, tested, not yet used by any scene.

**What fails**:

- **All in a room at 1.3 turns**: 61; a knock believes a flat wrong pose for ten
  frames.
- **All effects at 0.55 turns**: 84, under 89 before the hold.
- **Rotation on symmetric objects** says nothing: gear on a bench fell from 19
  to 0 within 2° while its surface error halved. A tooth off is right by ADD-S.
- **Cars**: 60 and 54, the first frames missed.
- **No box**: room 88 of 90, part on a table 36 of 36, **all in a room 0 of 90**.
- **Contour term**: room +3, but in a room −7, shelf −6. Left on its branch.
- **Fragile gains**, 0.7 and 0.3 on the turn fell to 11 of 90 on one path. Small
  changes move a run by a few frames: the room's 90 came from 86 with a sampling
  change of 4 %.
- **`cases` and the benchmark were not rerun**: the benchmark's code is unchanged;
  [the projection cases](projection-cases.md) are still those of 0.1.23.
