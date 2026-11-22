# Global initialisation

**[sketch]** When nothing is known of the pose, ICP alone fails: from a
random rotation it lands in a wrong minimum most of the time.

## Options, from the simplest

1. **Multistart.** Match the centroids, try $k$ rotations spread evenly over
   SO(3) (super-Fibonacci spirals, Alexa 2022), run ICP from each, keep the
   one that explains the most points. No descriptors, deterministic, easy to
   reason about. Needs a source covering most of the target, since a partial
   view moves the centroid.
2. **Principal axes.** Align the inertia axes: four candidates, very fast,
   wrong on near-symmetric shapes. A cheap first filter for multistart.
3. **Features and consensus.** FPFH descriptors, matched, then RANSAC or a
   certifiable solver (TEASER++). Handles partial overlap; more code, more
   parameters.
4. **Branch and bound.** Go-ICP: globally optimal, slow.

## [question] Which first

**[guess]** Multistart first: it is about a hundred lines, has no parameter
but $k$, and its failure (partial view) is the case features solve. Features
come second, built on the same k-d tree.

## [caution] Symmetry

A symmetric target has several right answers. A global method should report
when its best two candidates are both good and far apart, instead of
returning one at random.
