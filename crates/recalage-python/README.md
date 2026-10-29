# recalage for Python

Rigid registration of point clouds and meshes, from NumPy. Points are
`(n, 3)` arrays, transforms `(4, 4)` row-major arrays taking the source onto
the target. Defaults and errors are the Rust core's; the interpreter lock is
released while a target is built and while an alignment runs.

```python
import numpy as np
import recalage

grid = np.linspace(-1.0, 1.0, 30)
along, across = (axis.ravel() for axis in np.meshgrid(grid, grid))
model = np.column_stack((along, across, np.sin(3 * along) * np.cos(2 * across)))
target = recalage.Target(model, neighbors=12)
source = model - [0.03, -0.02, 0.01]
alignment = recalage.Icp().align(source, target)
print(alignment.transform.round(3), alignment.converged)
```

Three entry points, from the most to the least informed:

- `fit_rigid(source, target, weights=None)`: pairs are known, closed form.
- `Icp(...).align(source, target, initial=None)`: a rough pose is known.
- `Multistart(...).align(source, target)`: no pose at all.

A `Target` is built once and serves any number of alignments. Point-to-plane,
the default metric, needs target normals: give them, estimate them with
`neighbors`, or sample a mesh with `Target.from_mesh`.

## Development

```sh
just python-dev    # build into crates/recalage-python/.venv
just python-test   # ruff, then pytest
```
