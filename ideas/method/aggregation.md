# Aggregation

**[sketch]** The 2025 idea for variant V2, worth keeping as a question.

Under H2 ($\sigma \le \alpha\, d_{\min}$), the observations of one model
point form a tight cluster, far from the others. Clustering $\mathcal{Q}$
with a radius below $d_{\min}/2$ gives back **one representative per model
point**: the cluster mean, whose noise shrinks as $\sigma/\sqrt{N_i}$.
Small clusters are outliers.

What is left is variant V1 *without known order*: $n$ points against $n$
points, nearly noiseless.

## [question] Is it a contribution?

- It is a denoising step, known under other names (voxel or mean-shift
  downsampling). **[to check]** how close.
- What may be new is the **bound**: under H2, the error after aggregation
  and alignment can be predicted from $\sigma$, $\lambda$ and $n$, and
  checked on the generator.
- **[caution]** Real scans break H2 (denser than their noise). Aggregation
  then becomes plain downsampling, still useful for speed.
