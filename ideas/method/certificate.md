# A certificate

**[sketch]** A search returns a pose; a certificate says how far any pose can
be below it. It grades every method on a proved optimum, and the poses it
cannot exclude are the honest answer when several survive.

## The objective

The source points $x_i$, the target's points $Y$, a truncation $\tau$:

$$
f(R, t) = \frac{1}{n} \sum_i \min\bigl(d(R x_i + t, Y)^2, \tau^2\bigr)
$$

A [truncated least squares][tls], robust to outliers.
**[caution]** $Y$ is a finite set: on a mesh the proof is about its samples.

## The bound

**[to check]** [Branch and bound][bnb] over rotations as in [Go-ICP][goicp],
with its uncertainty radius.

Take a cube of rotation vectors of half-side $\sigma_r$ around $r_0$, of
translations of half-side $\sigma_t$ around $t_0$. Every pose in it moves $x_i$
by at most

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

## The argument in full

For a cube of centre $(r_0, t_0)$ and any pose $(r, t)$ in it:

1. **A turn is short.** The angle between $R(r)$ and $R(r_0)$ is at most
   $\lVert r - r_0 \rVert \le \sqrt 3 \sigma_r$: [the exponential map contracts
   distance from the identity](https://doi.org/10.1007/s11263-008-0186-9).
   **[to check]** Read again; a million drawn pairs in `certificate/proof.rs` agree.
2. **So a point moves little.** A turn by angle $\theta$ moves $x$ by
   $2 \sin(\theta / 2) \lVert x \rVert$, increasing up to $\theta = \pi$.
3. **Distance is 1-Lipschitz.** By the triangle inequality, moving a point by
   $\gamma$ changes its distance to any set by at most $\gamma$. So
   $d_i - \gamma_i$ is below the distance anywhere in the cube.
4. **The cost is monotone.** $\min(\max(u, 0)^2, \tau^2)$ grows with $u$, so the
   sum of step 3's values is below $f$ everywhere in the cube.

**[defined]** Each step holds for any set $Y$: the points, or the surface of a mesh
with `run_on_mesh`, where the proof is about the surface itself.

**[caution]** Trust: argued here, attacked by drawn poses, read by no outside
reviewer, in floating point with a slack of $10^{-9}$ of the reach.

## What it answers

- **A gap**: the given pose is at most this far above the optimum, in the
  domain.
- **The survivors**: the cubes that may still beat it, the motions the data
  does not exclude. A gear turned by a tooth should survive; a wrong pose
  should not.

## What it found

**[sketch]** On 32 instances of marks, [the page][page], asking 1 mm rms of
gap, 15 million cubes at most, 3 min 48 s and 0.7 GB for the whole page:

| Marks | Closed before, 3 million | Closed now, 15 million |
| --- | ---: | ---: |
| With strays | 0 of 16 | 16 of 16 |
| Without | 7 of 16 | 11 of 16 |

- **The largest stray instance needs 5.9 million cubes**, in seconds.
- **The 5 still open have no strays**, 12 or 30 marks at 3 mm: **[guess]** the
  same 1 mm is a far larger share of a small value.
- **A proof is not the truth**: on 12 marks with strays, the optimum is often
  far from the motion that made them. 3 strays in 12 leave poses as good.
  Only 4 of 16 stray instances, the bracket's 30 marks, end within 5°.
- **A tighter ask holds**: at 0.5 mm, 7 of 8 stray instances close, within 15
  million.

## Why strays did not close

**The upper bound, not the lower.** Three runs on the same 8 stray instances:

| Run | Closed | Budget |
| --- | ---: | ---: |
| As it was | 0 of 8 | 3 million |
| As it was, the truth given as a seed | 8 of 8 | 15 million |
| Polish from new centres, no truth given | 4 of 8 | 3 million |

- **A bound too high by a millimetre multiplies the tree without limit**: the
  cubes to split are those under the bound, and more of them have poses under
  it. With the bound above the optimum by more than the gap asked, the search
  cannot end.
- **The seeds were poor**: the methods graded are not robust to strays, and
  polishing restarted from the best pose found nothing new.

**[sketch]** How much to believe it: a sampled descent of the tree, keeping
40 000 cubes per level with a weight, predicts the size of a search without
running it. It gave 4.20 million where the run took 4 198 497.

## What was tried

| Idea | Measured | Verdict |
| --- | --- | --- |
| Polish from the 8 lowest centres of each batch | 0 to 4 of 8 at 3 million | Kept |
| Drop cubes wholly past a half turn | 0.6 times the cubes, same bound | Kept |
| Stop a sum past the ceiling | 24 to 15 s, same cubes | Kept |
| Inherit saturated points from the parent | Same cubes, 15 s against 16 | Dropped |
| Break ties on the value at the centre | Same closure, same bounds | Dropped |
| Nested translation, as in Go-ICP | 2.5 times the cubes, or far more | Dropped |

- **Polish starts**: 1 start closes the same 4 of 8, 8 reach a slightly lower
  value, 64 nothing more. Eight cost about what 56 cubes of a batch's 4096 do.
- **A half turn**: the cube of rotation vectors holds each rotation about
  twice. **[defined]** Every rotation has a vector of norm at most $\pi$.
- **Saturated points**: **[guess]** a cube with many of them is dropped soon by
  the stopped sum, which leaves them little to save. Being already monotone
  through the triangle inequality, they do not tighten the bound either.
- **Search order**: at a fixed upper bound, the cubes to split are the same in
  any order, and the run matched the sampled size (954 992 for 964 000). Order
  can only help find that bound sooner, so no depth-first dive was built.
  **[question]** Two scouts still ended high (8.10 to 7.89 mm): a better scout
  may save cubes there.

### Nested translation

**[sketch]** Measured with the sampled descent only, not built in the library:
a search per rotation cube over the whole box, stopping at the first witness.

- **Gear, 30 marks**: 2.35 million evaluations against 0.95 million.
- **Gear, 12 marks, and bracket, 30 marks**: a single level of rotation was not
  done in 3 minutes, where the whole tree in six dimensions takes seconds.
- **Why**: it kills rotation cubes one level sooner, an eighth of them, and
  then pays for the whole box again in each. Six dimensions inherit what a
  parent already excluded.
- **[caution]** Handing translations down from parent to child would remove
  that cost, and is close to the tree in six dimensions.

## Limits

- **[caution]** Floating point: no exact arithmetic. The bound is widened by a
  small slack, wider than rounding.
- **[caution]** A translation domain is given, and nothing outside it is
  claimed.
- **[caution]** The cost grows quickly with accuracy: past a budget the bound
  holds but stays open, and says so.
- **[caution]** Closed means no cube is left open: one settled at the finest
  size counts as done, and `lower` carries it.
- **[caution]** The margin is fixed from the scout's value: when the search then
  finds better, the gap proved is slightly over the one asked.
- **[guess]** Sizes: marks, tens of points; a scan must be thinned first.

[tls]: https://arxiv.org/abs/2001.07715
[bnb]: https://en.wikipedia.org/wiki/Branch_and_bound
[goicp]: https://arxiv.org/abs/1605.03344
[page]: ../../docs/benchmark/certified.md
