# Hypotheses

**[defined]** Three hypotheses of the 2025 statement. Each makes a variant
tractable; a method states which it needs.

## H1 — Exterior outliers

Outliers fall outside the convex hull of $\mathcal{P}$ (in the generator:
farther than a clearance from it). An outlier then never sits on the
surface, and a distance threshold separates it.

## H2 — Bounded isotropic noise

The noise is isotropic and small against the spacing of the model:
$\sigma \le \alpha \, d_{\min}$, with $d_{\min}$ the smallest distance
between two model points. Each observation then identifies the model point
it comes from.

## H3 — Dense enough model

The largest gap between neighbouring model points is at most $\Delta$. The
model is a sampled surface, not scattered points: a tangent plane exists
everywhere, and point-to-plane makes sense.

## [caution] Real data

- **H1** fails for a sensor: background, the table, the surgeon's glove sit
  next to the surface. The robust methods must not need it.
- **H2** fails for a scan denser than its noise (most depth cameras): use
  it only for aggregation ([aggregation](../method/aggregation.md)).
