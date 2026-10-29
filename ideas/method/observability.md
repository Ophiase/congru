# Observability

**[defined]** Measured on the generator: twelve noisy marks on a smooth,
rounded patch, aligned onto the mesh by point-to-plane ICP from a pose
within 14°. Every mark ends on the surface, the residual is below the
noise, and the rotation is 74° off. The same marks with their
correspondences known give the pose to the noise.

**On the surface is not in the right place.** A rounded patch lets the
points slide without leaving it; a plane lets them slide and spin; a
cylinder lets them roll. The residual cannot see any of it.

## What the library should say

**[sketch]** Point-to-plane already builds the 6×6 matrix
$H = \sum_i w_i\, J_i J_i^{\top}$ of the last step. Its eigenvectors with
small eigenvalues are the motions the data does not constrain: a rotation
about the centre of the patch, a slide along it. Returned with the
alignment, they turn a silent wrong answer into a stated one:

- **conditioning**: the ratio of the smallest to the largest eigenvalue;
- **free motions**: the eigenvectors below a threshold, as a rotation axis
  or a translation direction a person can read.

**[guess]** For marks, the predicted target registration error of
Fitzpatrick (1998) is the closed-form counterpart: from the noise and the
layout of the marks, the error to expect at a point of interest.

## [caution] Tests

A test that checks only the residual passes on this failure. Tests of the
library check the pose against the truth, or check that the free motions
are reported.
