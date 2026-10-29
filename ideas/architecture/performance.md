# Performance

**[sketch]** Where an ICP spends its time, and what to do about it.

1. **Nearest neighbours**: almost all of it. A k-d tree built once on the
   target (`kiddo`), queried in parallel over the source.
2. **Normals**: once per target, by fitting a plane to the k nearest
   neighbours; never in the loop.
3. **Solve**: a 3×3 SVD or a 6×6 Cholesky per iteration; negligible.

## Choices

- **Reuse the target.** Tracking a model over a video aligns many sources
  onto one target: the index and normals are built once.
- **Parallel by default, off by feature**, for embedded builds and for
  callers that already own their threads.
- **Downsample the source**, not the target: fewer queries, same accuracy
  past a density. **[to check]** on the benchmark.

## [question] The claim

"Fast" means nothing without a reference. **[guess]** Measure against Open3D
and small_gicp on the same data, and publish the script with the numbers.
