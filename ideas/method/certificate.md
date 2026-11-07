# A certificate

**[sketch]** A search returns a pose; a certificate says how far any pose can be
below it. It grades every method on a proved optimum, and the poses it cannot
exclude are the honest answer when several survive.

## The objective

The source points $x_i$, the target's points $Y$, a truncation $\tau$:

$$
f(R, t) = \frac{1}{n} \sum_i \min\bigl(d(R x_i + t, Y)^2, \tau^2\bigr)
$$

A [truncated least squares](https://arxiv.org/abs/2001.07715), robust to outliers.
**[caution]** $Y$ is a finite set: on a mesh the proof is about its samples.

## The bound

**[to check]** [Branch and bound](https://en.wikipedia.org/wiki/Branch_and_bound)
over rotations as in [Go-ICP](https://arxiv.org/abs/1605.03344), with its uncertainty
radius.

Take a cube of rotation vectors of half-side $\sigma_r$ around $r_0$, of translations
of half-side $\sigma_t$ around $t_0$. Every pose in it moves $x_i$ by at most

$$
\gamma_i = 2 s \lVert x_i \rVert + \sqrt 3 \sigma_t
$$

with $s = \sin\bigl(\min(\sqrt 3 \sigma_r / 2, \pi / 2)\bigr)$.

By the triangle inequality, with $d_i$ the distance at the cube's centre and
$e_i = \max(d_i - \gamma_i, 0)$:

$$
\min_{\text{cube}} f \ge \frac{1}{n} \sum_i \min(e_i^2, \tau^2)
$$

- **Upper bound**: the value of any pose, here the best evaluated.
- **Lower bound**: the least bound over the cubes still open.
- **The proof is the comparison**: their gap is measured, never assumed.

## What it answers

- **A gap**: the given pose is at most this far above the optimum, in the domain.
- **The survivors**: the cubes that may still beat it, the motions the data does
  not exclude. A gear turned by a tooth should survive; a wrong pose should not.

## What it found

**[sketch]** On 32 instances of marks, [the
page](../../docs/benchmark/certified.md):

- **It closes without strays, on enough marks**: 7 of 32, in about ten seconds,
  with three million cubes on twelve cores.
- **It never closes with strays**, 0 of 16: a truncated point weakens the bound
  most. This is the first thing to improve.
- **A small value is not the truth, proved**: on twelve clean marks a closed search
  can end far from the motion that made them. Few marks do not choose.

**[guess]** A tighter bound could reach the strays: a
[per-point uncertainty](https://arxiv.org/abs/1605.03344), or testing only the
marks that can still count.

## Limits

- **[caution]** Floating point: no exact arithmetic. The bound is widened by a small
  slack, wider than rounding.
- **[caution]** A translation domain is given, and nothing outside it is claimed.
- **[caution]** The cost grows quickly with accuracy: past a budget the bound holds
  but stays open, and says so.
- **[guess]** Sizes: marks, tens of points; a scan must be thinned first.
