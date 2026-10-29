# Observation model

**[defined]** The model of the 2025 attempt, kept as the definition of the
problem.

A model $\mathcal{P} = \{p_1, \ldots, p_n\} \subset \mathbb{R}^3$ is observed
through three corruptions, in order.

1. **Resampling.** Each $p_i$ yields $N_i$ observations, each drawn from
   $\mathcal{N}(p_i, \sigma^2 I)$. $N_i$ is $1$, a constant, or
   $\text{Poisson}(\lambda)$: a point may vanish or repeat.
2. **Outliers.** $U$ points uniform in a box $\mathcal{B}$ around the model,
   belonging to no $p_i$.
3. **Rigid motion.** Everything is moved by an unknown
   $(R, t) \in SO(3) \times \mathbb{R}^3$.

$$
\mathcal{Q} = \{\, R\,q + t \;:\; q \in \mathcal{S} \cup \mathcal{U} \,\},
\qquad
\mathcal{S} = \bigcup_{i=1}^{n} \{ s_{i,k} \sim \mathcal{N}(p_i, \sigma^2 I)
\}_{k=1}^{N_i}
$$

**Goal**: recover $(R, t)$, and which points of $\mathcal{Q}$ come from
which $p_i$.

## Added since

- **[sketch]** **Partial view**: before resampling, only the share of
  $\mathcal{P}$ on one side of a random plane is kept. Real sensors see one
  side; the surgical case sees a few percent
  ([origin](../origin.md)).
- **[sketch]** **Mesh model**: $\mathcal{P}$ may be a surface rather than
  points; resampling then draws from the surface
  ([mesh-targets](../method/mesh-targets.md)).

## Why this model

**[sketch]** It is generative: any method can be scored on it with the
truth in hand, and each corruption can be turned up alone until the method
breaks ([measuring-success](measuring-success.md)).
