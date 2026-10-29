# Measuring success

**[sketch]** A result is judged against the truth of the generator, with
the same measures for every method.

## Per alignment

- **Rotation error**: angle of $\hat R^{\top} R$, in degrees.
- **Translation error**: $\lVert \hat t - t \rVert$, in model units.
- **Target registration error**: distance moved, under the error, by a point
  of interest that was not used to align (the tip of a drill, the centre of
  a glenoid). **[to check]** the surgical literature reports this one
  ([prior art](../prior-art.md), Fitzpatrick 1998).
- **Residual** (RMSE of kept pairs) is *not* a measure of success: a wrong
  local minimum can have a small residual.

## Per method

- **Success rate**: share of seeds whose rotation error stays under a
  threshold (for example 2°).
- **Breaking point**: the noise, the outlier share, the overlap or the
  initial angle at which the success rate falls below 90 %. One curve per
  corruption, everything else clean.
- **Time** per alignment, by point count, on one core and on all.

## [caution]

A method tuned on the generator learns the generator. Real scans, with their
licences recorded, come before any claim.
