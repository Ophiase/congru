# Bindings

**[sketch]** Every language sees the same three entry points
([what we know](../problem/what-we-know.md)), with the core's defaults.

## Python

- PyO3 and maturin; NumPy `(n, 3)` arrays in, a `(4, 4)` array out, no
  copy when the array is already contiguous `float64`.
- The interpreter lock is released during an alignment.
- Type stubs shipped, so an editor completes the parameters.
- **[sketch]** `crates/recalage-python`: a native `recalage._recalage` under
  a pure-Python `recalage`. A wrong array shape is a `ValueError`; anything
  the core refuses is a `RegistrationError`, a `ValueError` carrying the
  core's message. A weak motion's kind is a `Rotation` or a `Translation`,
  matched with `match`.
- **[sketch]** Built against the stable ABI of Python 3.10, so `cargo clippy`
  works with any interpreter from 3.10 on; the package itself asks for 3.12.
- **[question]** one wheel per platform through CI, from the first release?

## C and C++

- A C ABI: plain structs of options, status codes, an opaque handle for a
  reusable target, row-major `double[16]` for transforms.
- A header-only C++ wrapper over it: RAII for the handle, `std::span` in,
  exceptions or `std::expected` out. **[question]** which error style?
- A panic never crosses the boundary: it becomes a status code.
- **[guess]** the header is written by hand and checked by a compiled
  example, rather than generated: it is short, and it is the contract.

## [caution]

A binding that adds a default, a conversion or a rule of its own drifts from
the core; the skins only translate.
