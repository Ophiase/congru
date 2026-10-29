# Benchmark

**[sketch]** The benchmark is the part of the project most likely to matter
beyond its code: few libraries say where their method breaks.

## Design

- **Data**: the observation model on analytic shapes without symmetry
  first; then real scans with their licences recorded.
- **Sweeps**: one corruption at a time (noise, outlier share, overlap,
  initial angle), the rest clean; many seeds per point.
- **Methods**: every entry point of the library with its defaults, and the
  same sweeps for Open3D and small_gicp as references.
- **Outputs**: success rate and error against each corruption, time against
  point count. Figures drawn from the data, never by hand.

## [question] Where it lives

A CLI command writing CSV, and figures drawn natively from that CSV for the
README and a paper in Typst. The figures of 2025 are retired.
