# The hypothesis ladder

**[defined]** Every hypothesis buys speed or simplicity; the project states
each one, makes the method that relies on it excellent, then offers the
next rung for when it fails.

| Hypothesis | What it buys | When it fails | The next rung |
| --- | --- | --- | --- |
| **Rigid** | Six numbers to find, closed forms, certifiable answers | Flexible parts, assemblies, a robot arm in view | A few bending modes; then a kinematic chain of rigid parts |
| **Mostly the object** (an upstream mask or crop isolated it) | Least squares and mild kernels suffice | No segmentation: a headset sees the whole room | Robust kernels and trimming; then consensus (RANSAC, TEASER-like) that tolerates a majority of outliers |
| **Mostly visible** | Centroids and global shape are reliable | Occlusion, a narrow field of view | Trimming and a motion prior; then partial-to-full matching by local features |
| **Near guess** | Local ICP: fast, accurate | First frame, a knock, a lost track | Multistart over SO(3); then descriptors and consensus |
| **Calibrated sensor** | Points are where the sensor says | Lens distortion, rolling shutter, a shaking mount | Robust kernels absorb a little; then distortion as an unknown |
| **Dense enough** | Normals estimated reliably | Cheap sensors, far objects, a few marks | Point-to-point and the closed form; then [observability](../method/observability.md) to say what is free |

**[defined]** A method's documentation names the rungs it needs, and the
[benchmark](../../docs/benchmark/protocol.md) has a situation that breaks
each one, so the cost of dropping a hypothesis is measured, not guessed.

**[sketch]** Order of work: excellence on the rigid, segmented, near case
first (milestone 1); then outlier-majority scenes without a mask, the case
of a headset seeing a whole room; then non-rigid, by modes and kinematics.
