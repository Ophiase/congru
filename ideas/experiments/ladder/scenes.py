from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray

Points = NDArray[np.float64]

CAMERA = np.array([0.0, 0.6])
RAY_COUNT = 64
HALF_FIELD = np.radians(24.0)
NOISE = 0.004

BRACKET = np.array(
    [[-0.14, -0.07], [0.14, -0.07], [0.14, -0.02], [-0.06, -0.02], [-0.06, 0.09], [-0.14, 0.09]]
)
TABLE = np.array([[-0.45, -0.1], [0.45, -0.1], [0.45, -0.07], [-0.45, -0.07]])
TWIN = BRACKET * [-0.9, 0.9] + [0.3, -0.007]
CAN = np.array([[-0.13, 0.17], [0.0, 0.17], [0.0, 0.25], [-0.13, 0.25]])


@dataclass(frozen=True)
class Step:
    name: str
    around: bool = False
    noisy: bool = True
    strays: int = 0
    others: tuple[Points, ...] = ()
    turned: float = 0.0


LADDER = (
    Step("seen-all-round", around=True, noisy=False),
    Step("noise", around=True),
    Step("one-side", noisy=True),
    Step("strays", strays=12),
    Step("table", others=(TABLE,)),
    Step("twin", others=(TABLE, TWIN)),
    Step("can", others=(TABLE, CAN)),
    Step("flat-side", others=(TABLE,), turned=np.pi),
)


def rotation(angle: float) -> Points:
    return np.array([[np.cos(angle), -np.sin(angle)], [np.sin(angle), np.cos(angle)]])


def placed(polygon: Points, angle: float, shift: Points) -> Points:
    return polygon @ rotation(angle).T + shift


def resting_shift(step: Step) -> Points:
    lowest = placed(BRACKET, step.turned, np.zeros(2))[:, 1].min()
    return np.array([0.0, BRACKET[:, 1].min() - lowest])


def outline_samples(polygon: Points, spacing: float = 0.004) -> Points:
    ends = np.roll(polygon, -1, axis=0)
    return np.vstack(
        [
            np.linspace(start, end, max(2, int(np.linalg.norm(end - start) / spacing)), endpoint=False)
            for start, end in zip(polygon, ends, strict=True)
        ]
    )


def ray_directions() -> Points:
    angles = np.linspace(-HALF_FIELD, HALF_FIELD, RAY_COUNT)
    return np.stack((np.sin(angles), -np.cos(angles)), axis=1)


def depths_along_rays(polygons: tuple[Points, ...]) -> NDArray[np.float64]:
    directions = ray_directions()
    nearest = np.full(len(directions), np.inf)
    for polygon in polygons:
        starts = polygon
        edges = np.roll(polygon, -1, axis=0) - polygon
        for ray, direction in enumerate(directions):
            offset = starts - CAMERA
            denominator = direction[0] * edges[:, 1] - direction[1] * edges[:, 0]
            with np.errstate(divide="ignore", invalid="ignore"):
                along = (offset[:, 0] * edges[:, 1] - offset[:, 1] * edges[:, 0]) / denominator
                across = (offset[:, 0] * direction[1] - offset[:, 1] * direction[0]) / denominator
            valid = np.isfinite(along) & (along > 0.0) & (across >= 0.0) & (across <= 1.0)
            if valid.any():
                nearest[ray] = min(nearest[ray], along[valid].min())
    return nearest


def sensed(step: Step, generator: np.random.Generator) -> tuple[Points, NDArray[np.float64] | None]:
    part = placed(BRACKET, step.turned, resting_shift(step))
    if step.around:
        samples = outline_samples(part)[::9]
        noise = generator.normal(0.0, NOISE if step.noisy else 0.0, samples.shape)
        return samples + noise, None
    depths = depths_along_rays((part, *step.others))
    if step.noisy:
        depths = depths + generator.normal(0.0, NOISE, depths.shape)
    seen = np.isfinite(depths)
    points = CAMERA + ray_directions()[seen] * depths[seen, None]
    strays = generator.uniform([-0.25, -0.07], [0.25, 0.3], (step.strays, 2))
    return np.vstack((points, strays)), depths
