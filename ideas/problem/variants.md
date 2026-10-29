# Variants

**[defined]** The three variants of 2025, ordered by difficulty.
**[sketch]** Each paired with the method that answers it.

| Variant | Observation | Method |
| --- | --- | --- |
| **V1 — one to one** | $N_i = 1$, no outlier, order known | [closed form](../method/closed-form.md) |
| **V2 — aggregated** | Known $d_{\min}$, H2, a few outliers | [aggregation](../method/aggregation.md), then V1 |
| **V3 — general** | Any $N_i$, outliers, partial view | [ICP family](../method/icp-family.md), [robustness](../method/robustness.md), [initialisation](../method/global-initialisation.md) |

## The axis that matters more

**[sketch]** The variants grade the *noise*. What changes the method more is
**what is known beforehand** ([what-we-know](what-we-know.md)): the
correspondences, a rough pose, or nothing. V1 with unknown order is already
V3.

**[question]** Which to build first: the marker case (few marks, a mesh, a
rough pose) or the scan case (dense, no pose)? See [open](../open.md).
