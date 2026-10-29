# Mesh targets

**[sketch]** In the founding case, the target is a mesh: a model of the
bone. Two ways to measure the distance of a point to it.

| Way | How | For | Against |
| --- | --- | --- | --- |
| **Sampling** | Spread points over the faces, each with its face normal; align onto them with point-to-plane | Reuses the k-d tree; nothing new to write | An error of about the sampling spacing |
| **Exact** | Closest point on a triangle, through a bounding-volume hierarchy | Exact distance, no spacing to choose | A second spatial index to write and keep fast |

**[guess]** Sampling first, with a spacing below the noise of the marks:
point-to-plane against dense samples is close to the true distance to the
surface. The exact way, if the benchmark shows the spacing matters.

## [caution]

The samples must be deterministic: the same mesh always gives the same
points, or two runs of a surgical alignment disagree for no reason.
