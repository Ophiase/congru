# Prior art

**[to check]** every entry: the references are from memory and must be read
again before the paper cites them.

## Closed form, known correspondences

- Kabsch, *Acta Crystallographica*, 1976: the rotation by SVD.
- Horn, *JOSA A*, 1987: the same through unit quaternions.
- Umeyama, *IEEE TPAMI*, 1991: the reflection fix, and scale.
- Fitzpatrick, West and Maurer, *IEEE TMI*, 1998: predicting the target
  registration error from fiducial noise and layout; the reference for the
  marker case.

## Local refinement

- Besl and McKay, *IEEE TPAMI*, 1992: ICP, point-to-point.
- Chen and Medioni, *Image and Vision Computing*, 1992: point-to-plane.
- Zhang, *IJCV*, 1994: distance thresholds for outliers.
- Rusinkiewicz and Levoy, *3DIM*, 2001: the taxonomy of ICP variants.
- Chetverikov et al., *ICPR*, 2002: trimmed ICP, for partial overlap.
- Segal, Hähnel and Thrun, *RSS*, 2009: generalised ICP (plane-to-plane).
- Babin, Giguère and Pomerleau, *ICRA*, 2019: robust kernels compared.
- Chebrolu et al., *RA-L*, 2021: adaptive robust kernels.

## Global

- Rusu, Blodow and Beetz, *ICRA*, 2009: FPFH descriptors.
- Zhou, Park and Koltun, *ECCV*, 2016: fast global registration.
- Yang et al., *IEEE TPAMI*, 2016: Go-ICP, globally optimal by
  branch and bound.
- Yang, Shi and Carlone, *IEEE T-RO*, 2021: TEASER++, certifiable with
  99 % outliers.
- Alexa, *CVPR*, 2022: super-Fibonacci spirals, even samples of SO(3).

## Probabilistic

- Myronenko and Song, *IEEE TPAMI*, 2010: coherent point drift; closest in
  spirit to the observation model of [problem/](problem/observation-model.md).

## Software

- **Open3D**, **PCL**, **libpointmatcher**: complete toolkits, C++ first.
- **small_gicp** (Koide, *JOSS*, 2024) and **KISS-ICP** (Vizzo et al.,
  *RA-L*, 2023): fast, focused, the bar to measure against.
- **Rust**: no maintained registration crate known to the author; kd-trees
  (`kiddo`) and linear algebra (`nalgebra`) exist.

## Survey

- Pomerleau, Colas and Siegwart, *Foundations and Trends in Robotics*, 2015.
