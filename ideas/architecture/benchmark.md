# Benchmark

**[defined]** The benchmark is the part of the project most likely to matter
beyond its code: few libraries say where their method breaks. Its protocol
is [docs/benchmark/protocol.md](../../docs/benchmark/protocol.md), its
tables [docs/benchmark/results.md](../../docs/benchmark/results.md).

## Decisions

- **One contract**: every method is an [`Aligner`](contract.md), so one loop
  runs them all and a new method costs one line in the catalogue.
- **Situations from the hypotheses**: each breaks one of the
  [ladder](../problem/hypothesis-ladder.md), one at a time, then all
  together.
- **Shares of the model's size**, so a situation means the same on every
  model.
- **Two seed pools**: tuning and evaluation, disjoint; the published tables
  never guide a choice.
- **Confidently wrong** is reported beside success: a robot is hurt more by
  a trusted wrong pose than by an admitted failure.
- **Wilson intervals** on every success rate, so thirty seeds are not read
  as a certainty.

## [question] Still open

- Comparison with Open3D and small_gicp, on the same frames written to
  disk.
- Real scans, once a source with a clear licence is found
  ([catalogue/sources.yaml](../../catalogue/sources.yaml)).
