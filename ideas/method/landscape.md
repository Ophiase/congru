# The landscape of methods

**[sketch]** Rigid registration is not one algorithm. It is a family of
answers to one question, and which answer wins depends on what is known
and what the data does. congru treats ICP as one family among several:
the default only where the [benchmark](../architecture/benchmark.md) says
so, and nowhere by principle.

## With correspondences

The pairs are known, or proposed by a matcher.

| Method | Idea | In congru |
| --- | --- | --- |
| [Kabsch] | The rotation from the SVD of the cross-covariance | `fit_rigid` |
| [Umeyama], [Procrustes] | The same with scale; the determinant controlled to forbid a reflection | Rotation only, **[question]** scale |
| [RANSAC], LO-RANSAC | Fit on minimal random subsets, keep the largest consensus, refine locally | Inside `Consensus`, checked on the whole shape |
| [TEASER++] | Truncated least squares, decoupled scale, rotation and translation, certified | **[to check]** a horizon |

## Without correspondences, local

A pose is roughly known; pairs are guessed from proximity and refined.

| Method | Idea | In congru |
| --- | --- | --- |
| [ICP], point-to-point | Nearest neighbours, then Kabsch, repeat | `Metric::PointToPoint` |
| ICP, [point-to-plane] | The residual along the target normal; converges fast on surfaces | `Metric::PointToPlane` |
| [Generalised ICP], plane-to-plane | Both surfaces' local covariances in the residual | **[question]** |
| Robust ICP, [M-estimators] | Huber, Cauchy, Geman–McClure, Tukey weights | `Kernel` |
| Trimmed ICP | Only the closest share of pairs | `Icp::overlap` |
| Annealed ICP | A kernel scale shrinking from coarse to fine | `Annealed` |
| Adaptive ICP | Kernel and gate set from the noise measured at each stage | `Adaptive` |
| Pyramid ICP | Downsampled first, full resolution last | **[sketch]** |
| [NDT] | The target as a grid of Gaussians; the source scored against them | **[question]** |
| [CPD] | The source as a Gaussian mixture moving toward the target; also non-rigid | **[question]** |

## Without correspondences, global

Nothing is known of the pose.

| Method | Idea | In congru |
| --- | --- | --- |
| Multistart | Local ICP from rotations spread over SO(3), the best kept | `Search`, `Guarded` |
| [FPFH] and RANSAC | Local descriptors matched, then a consensus | `Consensus` |
| Fast global registration | Descriptors and a graduated non-convex objective | **[to check]** |
| [Go-ICP] | Branch and bound over SE(3): globally optimal, slow | **[to check]** |

## Learned

| Method | Idea | In congru |
| --- | --- | --- |
| [PointNet], PointNet++, DGCNN | Permutation-invariant features, local neighbourhoods | Out of scope for the core |
| [Deep Closest Point], [GeoTransformer], RegFormer | Learned correspondences, then a closed form | **[question]** as an `Aligner` behind a feature |

## Around registration

- **Spatial indices**: k-d trees (in congru), octrees, voxel hashes.
- **Clustering**: [DBSCAN] separates the object from clutter when no mask
  exists; the first rung of dropping the *mostly the object* hypothesis
  ([ladder](../problem/hypothesis-ladder.md)).
- **Tracking**: a motion filter around any of the above
  ([tracking](tracking.md)).

**[defined]** Every method that lands is an [`Aligner`](../architecture/contract.md)
and gets its row in every table of the benchmark.

[Kabsch]: https://en.wikipedia.org/wiki/Kabsch_algorithm
[Umeyama]: https://doi.org/10.1109/34.88573
[Procrustes]: https://en.wikipedia.org/wiki/Orthogonal_Procrustes_problem
[RANSAC]: https://en.wikipedia.org/wiki/Random_sample_consensus
[TEASER++]: https://arxiv.org/abs/2001.07715
[ICP]: https://en.wikipedia.org/wiki/Iterative_closest_point
[point-to-plane]: https://doi.org/10.1016/0262-8856(92)90066-C
[Generalised ICP]: https://www.roboticsproceedings.org/rss05/p21.pdf
[M-estimators]: https://en.wikipedia.org/wiki/M-estimator
[NDT]: https://en.wikipedia.org/wiki/Normal_distributions_transform
[CPD]: https://arxiv.org/abs/0905.2635
[FPFH]: https://doi.org/10.1109/ROBOT.2009.5152473
[Go-ICP]: https://arxiv.org/abs/1605.03344
[PointNet]: https://arxiv.org/abs/1612.00593
[Deep Closest Point]: https://arxiv.org/abs/1905.03304
[GeoTransformer]: https://arxiv.org/abs/2202.06688
[DBSCAN]: https://en.wikipedia.org/wiki/DBSCAN
