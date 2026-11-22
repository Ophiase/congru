# What we know beforehand

**[sketch]** The library's entry points follow what the caller knows, not
the name of an algorithm.

| The caller knows | Entry point | Cost |
| --- | --- | --- |
| Which point goes with which | `fit_rigid` — [closed form](../method/closed-form.md) | One SVD |
| A pose within about 20° | `Icp` — [ICP family](../method/icp-family.md) | Some nearest-neighbour passes |
| Nothing | `Multistart` or features — [initialisation](../method/global-initialisation.md) | Many ICPs, or descriptors |

## Other knowledge worth a parameter

- **Noise level** $\sigma$: sets the scale of a robust kernel, and the
  convergence tolerance.
- **Overlap**: the share of the source that sees the target; sets trimming.
- **Normals**: given by a mesh or a sensor, otherwise estimated.
- **Symmetry**: a symmetric target has several right answers; the library
  should say so rather than pick one.
  **[question]** report all minima within a tolerance?
