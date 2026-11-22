# Closed form

**[defined]** When the correspondences are known (variant V1, fiducials,
marks matched by their colour or number), the best rigid motion in least
squares has a closed form: no iteration, no initial pose.

$$
\min_{R \in SO(3),\, t} \sum_i w_i \,\lVert R\,s_i + t - q_i \rVert^2
$$

Centre both sets on their weighted centroids, take the SVD of the
cross-covariance $H = U \Sigma V^{\top}$, then
$R = V \operatorname{diag}(1, 1, \det V U^{\top})\, U^{\top}$ and
$t = \bar q - R \bar s$ (Kabsch, Umeyama).

## What the library adds

- **[sketch]** Weights, so a doubtful mark counts less.
- **[sketch]** A refusal on collinear points, where the rotation about the
  line is free; never a silent arbitrary answer.
- **[guess]** The predicted target registration error from the noise and the
  layout of the marks (Fitzpatrick 1998), so an operator knows before trusting
  the result. This may be the most useful single feature for the founding
  case.

## [caution]

The determinant correction is what keeps a reflection out. Code that skips
it passes tests on clean data and fails on noisy, nearly planar marks.
