from typing import ClassVar, final

import numpy as np
from numpy.typing import ArrayLike, NDArray

type Points = ArrayLike
type Indices = ArrayLike
type Matrix = NDArray[np.float64]
type Vector = NDArray[np.float64]

class RegistrationError(ValueError): ...

@final
class Metric:
    POINT_TO_POINT: ClassVar[Metric]
    POINT_TO_PLANE: ClassVar[Metric]
    def __int__(self) -> int: ...

@final
class Kernel:
    @staticmethod
    def quadratic() -> Kernel: ...
    @staticmethod
    def huber(scale: float) -> Kernel: ...
    @staticmethod
    def cauchy(scale: float) -> Kernel: ...
    @staticmethod
    def geman_mcclure(scale: float) -> Kernel: ...
    @staticmethod
    def tukey(scale: float) -> Kernel: ...
    def weight(self, residual: float) -> float: ...

@final
class Target:
    def __new__(
        cls,
        points: Points,
        *,
        normals: Points | None = None,
        neighbors: int | None = None,
    ) -> Target: ...
    @staticmethod
    def from_mesh(vertices: Points, triangles: Indices, spacing: float) -> Target: ...
    @property
    def points(self) -> Matrix: ...
    @property
    def normals(self) -> Matrix | None: ...

@final
class Icp:
    def __new__(
        cls,
        *,
        metric: Metric = ...,
        kernel: Kernel = ...,
        max_correspondence_distance: float | None = ...,
        overlap: float = ...,
        max_iterations: int = ...,
    ) -> Icp: ...
    def align(
        self,
        source: Points,
        target: Target,
        initial: ArrayLike | None = None,
    ) -> Alignment: ...
    @property
    def metric(self) -> Metric: ...
    @property
    def kernel(self) -> Kernel: ...
    @property
    def max_correspondence_distance(self) -> float | None: ...
    @property
    def overlap(self) -> float: ...
    @property
    def max_iterations(self) -> int: ...

@final
class Multistart:
    def __new__(cls, *, starts: int = ..., icp: Icp = ...) -> Multistart: ...
    def align(self, source: Points, target: Target) -> Alignment: ...
    @property
    def starts(self) -> int: ...
    @property
    def icp(self) -> Icp: ...

@final
class Fit:
    @property
    def transform(self) -> Matrix: ...
    @property
    def rmse(self) -> float: ...

def fit_rigid(
    source: Points,
    target: Points,
    weights: ArrayLike | None = None,
) -> Fit: ...

@final
class Alignment:
    @property
    def transform(self) -> Matrix: ...
    @property
    def rmse(self) -> float: ...
    @property
    def inlier_fraction(self) -> float: ...
    @property
    def iterations(self) -> int: ...
    @property
    def converged(self) -> bool: ...
    @property
    def observability(self) -> Observability | None: ...

@final
class Observability:
    @property
    def conditioning(self) -> float: ...
    @property
    def weak_motions(self) -> tuple[WeakMotion, ...]: ...

@final
class WeakMotion:
    @property
    def stiffness(self) -> float: ...
    @property
    def kind(self) -> Rotation | Translation: ...

@final
class Rotation:
    __match_args__ = ("axis", "through")
    @property
    def axis(self) -> Vector: ...
    @property
    def through(self) -> Vector: ...

@final
class Translation:
    __match_args__ = ("direction",)
    @property
    def direction(self) -> Vector: ...
