# Install and quick start

congru is at 0.1.x: the core works and is measured, the API is not frozen,
and nothing is published yet. The package managers come when the first users
need them; today, build from the repository.

## Today

### Rust

From a checkout, a path dependency is the shortest route:

```toml
[dependencies]
congru = { path = "crates/congru" }
```

From another repository, point Cargo at the git URL; it finds the `congru`
package inside the workspace:

```toml
[dependencies]
congru = { git = "https://github.com/Ophiase/congru" }
```

```rust
use congru::{Icp, PointCloud, RigidTransform, Target};

let target = Target::with_estimated_normals(PointCloud::from_xyz(&model)?, 10)?;
let source = PointCloud::from_xyz(&scan)?;
let alignment = Icp::default().align(&source, &target, &RigidTransform::identity())?;
```

### Python

The extension builds with maturin, so a Rust toolchain is needed. The
package lives in `crates/congru-python`, so a git URL names that
subdirectory:

```sh
uv add "congru @ git+https://github.com/Ophiase/congru#subdirectory=crates/congru-python"
```

Or, in `pyproject.toml`:

```toml
[project]
dependencies = ["congru"]

[tool.uv.sources.congru]
git = "https://github.com/Ophiase/congru"
subdirectory = "crates/congru-python"
```

The same direct URL works with pip:

```sh
pip install "congru @ git+https://github.com/Ophiase/congru#subdirectory=crates/congru-python"
```

Inside a checkout, `just python-dev` builds the extension into a virtual
environment instead.

```python
import congru

target = congru.Target(model, neighbors=12)
alignment = congru.Icp().align(scan, target)
print(alignment.transform)
```

### C and C++

```sh
cargo build --release -p congru-capi
```

Link `target/release/libcongru_capi.a` and include
`crates/congru-capi/include/congru.h` or `congru.hpp`. The ABI is unstable
until 1.0.

## Incoming

When the project leaves 0.1.x:

- `cargo add congru` (crates.io)
- `pip install congru` (PyPI)
- A versioned C ABI, with a compatibility policy
- A published benchmark with every model and every seed

Until then, pin a commit: the API may move.
