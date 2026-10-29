# Stack

**[sketch]** The stack is part of the design: each piece is the best of its
kind for one job, and the core carries as few as it can. A dependency is a
promise kept by someone else; the core makes four.

## The core

| Piece | Why this one |
| --- | --- |
| **Rust 2024** | Speed of C, types that carry the rules, one artefact for every skin |
| **nalgebra** | Fixed-size 3×3 and 6×6 algebra on the stack, SVD and Cholesky; the lingua franca of Rust geometry |
| **kiddo** | The fastest k-d tree in Rust, built once and immutable, SIMD where it can |
| **rayon** | Data parallelism without a thread in sight; behind a feature, for embedded builds |
| **thiserror** | Errors as an enum the caller can match, nothing more |

**[caution]** `faer` is faster on large matrices; registration only ever
solves 3×3 and 6×6 systems, where nalgebra's fixed sizes win.

## The skins

| Skin | Stack |
| --- | --- |
| Python | PyO3 and maturin; NumPy arrays without copy; uv, ruff and pytest around it; type stubs |
| C | A hand-written header, the contract, checked by a compiled example |
| C++ | A header-only C++20 wrapper over the C ABI: `std::span` in, RAII for handles |
| Command line | clap, JSON out, PLY and XYZ in |

## Around the code

| Need | Tool |
| --- | --- |
| Recipes | `just`, every recipe with its `[doc]` |
| Markdown, TOML | `rumdl`, `taplo` |
| Licences, advisories | `cargo-deny` |
| Figures and animations | Computed by the library itself, written as SVG and GIF from Rust: no plotting stack, nothing drawn by hand |
| Paper | Typst, the figures included from the same files |
| Continuous integration | GitHub Actions, the same `just` recipes as on a laptop |

## [caution] What stays out

No Python in the core path, no C++ build system to install, no GPU
dependency, no network. Each of those would make the brick harder to put
inside a robot.
