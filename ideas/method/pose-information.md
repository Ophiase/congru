# What a pose is worth: the information of its support

**[sketch]** An answer to one question of [evidence along
rays](ray-evidence.md): is "no flat face flush with a wall" a sensor fact or a
physical prior? Here, a sensor fact, as long as the hits are counted for what
they say.

## The author's sentence

Inside the zone the rays leave free, take the place with the most model on its
border, "that is, with information". Measured with `congru-demo rank`, the two
halves are not equal. The count alone fails: the part flipped, its plate flush
on the turntable, has more hits than the truth.

**A hit counts for what it says about the pose.** A flat face, however large,
says nothing of sliding along itself.

## The score

The hits of [the visibility problem](../problem/visibility.md) with their area
$a(u) = (m_T(u)/f)^2$, $f$ the focal length, and $J_u$ the [point-to-plane] row
of the rendered surface at pixel $u$, rotations scaled by the spread of the hits:

$$
H(T) = \sum_{u \in \text{hit}} a(u)\, J_u J_u^\top,
\qquad
S(T) = \Big(\prod_{k=1}^{6} \lambda_k(H)\Big)^{1/6}
       - \sum_{u \in \text{through}} a(u)
$$

The geometric mean of the eigenvalues is the [D-optimal] criterion of experiment
design, and $H$ the matrix [observability](observability.md) already reads.

Three things the measures forced:

- **A hit needs the surface's direction too.** A thin side of the part, across
  the turntable, lies within the tolerance of its depth and looks like
  information about sliding. A hit counts only if the image's normal, a plane
  fitted over 7 × 7 pixels, agrees with the model's within 45°.
- **A ray through needs a neighbourhood.** A sensor mixes both sides of an edge:
  on the rim of the true silhouette the image seems to see through the model. A
  pixel refutes only if all returns two pixels around it lie behind the model.
- **The tolerance is half the finder's step**, 6 mm for a noise of about 5 mm.

## Measured

**[sketch]** Demo B, one sensor seed. Frames where the truth outscores every wrong
pose (more than 5° or 2 cm off), among the votes refined along rays and 100 random
poses. The 64 best of these are then improved by a local search on the score
itself. The frames were fixed in advance: 12 to choose on, 18 kept apart.

| Score | Room | In a room | Sensor |
| --- | --- | --- | --- |
| Agreement × share in frame | 9 / 10 | 7 / 10 | 10 / 10 |
| $S$ at the finder's step | 10 / 10 | 7 / 10 | 10 / 10 |
| $S$ at half of it | 10 / 10 | 9 / 10 | 10 / 10 |

Kept apart only, $S$ at half: 6 / 6, 5 / 6, 6 / 6.

**[caution]** The same run without the normals, a stronger adversary (every start
improved), interrupted: 6 / 8 and 7 / 10 in the room conditions; agreement 1 / 9
and 3 / 10. So the numbers of the table depend on the adversary; the order did
not change. Choosing among three variants on these frames is itself a little
tuning.

[D-optimal]: https://en.wikipedia.org/wiki/Optimal_experimental_design
[point-to-plane]: https://en.wikipedia.org/wiki/Iterative_closest_point
