from dataclasses import dataclass

import numpy as np
from scipy.spatial import cKDTree

from camera import NOISE_AT_ONE_METRE
from meshes import FloatArray, Pose
from solver import draw, read_frame

SENSOR_FLOOR = 0.004
ZONE_DEVIATIONS = 3.0
AGREEMENT_TOLERANCE = 0.0125
HEALTH_GATE = 0.03
EDGE_GAP = 2
RIM_FACING = 0.35
RIM_STEP = 0.03


def deviation(depth: FloatArray) -> FloatArray:
    return np.hypot(NOISE_AT_ONE_METRE * depth * depth, SENSOR_FLOOR)


def edge_depth(rendered: FloatArray) -> FloatArray:
    filled = np.nan_to_num(rendered, nan=np.inf)
    neighbours = [
        np.roll(filled, shift, axis)
        for axis, shift in ((0, EDGE_GAP), (0, -EDGE_GAP), (1, EDGE_GAP), (1, -EDGE_GAP))
    ]
    return np.stack(neighbours).min(axis=0)


@dataclass(frozen=True)
class PointVerdict:
    drawn: int
    facing: int
    hits: int
    shadow: int
    through: int
    silent: int
    standing: int
    flush: int

    @property
    def through_share(self) -> float:
        return self.through / max(self.drawn - self.silent, 1)

    @property
    def hit_share(self) -> float:
        return self.hits / max(self.facing, 1)

    @property
    def outline_share(self) -> float:
        return self.standing / max(self.standing + self.flush, 1)

    @property
    def score(self) -> float:
        return (self.hit_share - self.through_share) * self.outline_share


def judge_points(
    depth: FloatArray,
    pose: Pose,
    points: FloatArray,
    normals: FloatArray,
    budget: int,
    generator: np.random.Generator,
) -> PointVerdict:
    drawn = draw(len(points), budget, generator)
    placed, turned = pose.apply(points[drawn]), pose.turn(normals[drawn])
    facing = (placed * turned).sum(axis=1) < 0.0
    seen = read_frame(depth, placed)
    silent = ~np.isfinite(seen)
    gap = np.where(silent, 0.0, seen - placed[:, 2])
    tolerance = ZONE_DEVIATIONS * deviation(placed[:, 2])
    near = np.abs(gap) <= tolerance
    rim = near & ~silent & (np.abs((placed * turned).sum(axis=1)) < RIM_FACING * placed[:, 2])
    outer = placed[rim] + turned[rim] * RIM_STEP * placed[rim, 2, None]
    outer_seen = read_frame(depth, outer)
    outer_gap = outer_seen - outer[:, 2]
    return PointVerdict(
        drawn=len(drawn),
        facing=int((facing & ~silent).sum()),
        hits=int((near & facing & ~silent).sum()),
        shadow=int((gap < -tolerance).sum()),
        through=int((gap > tolerance).sum()),
        silent=int(silent.sum()),
        standing=int((~np.isfinite(outer_seen) | (outer_gap > AGREEMENT_TOLERANCE)).sum()),
        flush=int((np.abs(outer_gap) <= AGREEMENT_TOLERANCE).sum()),
    )


def current_agreement(depth: FloatArray, rendered: FloatArray) -> float:
    covered = np.isfinite(rendered)
    seen, expected = depth[covered], rendered[covered]
    agree = (np.abs(seen - expected) <= AGREEMENT_TOLERANCE).sum()
    behind = (seen > expected + AGREEMENT_TOLERANCE).sum()
    inside = (agree - behind) / max(int(covered.sum()), 1)
    edge = edge_depth(rendered)
    outside = ~covered & np.isfinite(edge)
    outer_seen, outer_edge = depth[outside], edge[outside]
    standing = (np.isnan(outer_seen) | (outer_seen > outer_edge + AGREEMENT_TOLERANCE)).sum()
    flush = (np.abs(outer_seen - outer_edge) <= AGREEMENT_TOLERANCE).sum()
    return float(inside * standing / max(int(standing + flush), 1))


def health_inliers(
    cloud: FloatArray,
    centre: FloatArray,
    reach: float,
    visible_model: FloatArray,
    budget: int,
    generator: np.random.Generator,
) -> float:
    crop = cloud[np.linalg.norm(cloud - centre, axis=1) <= reach]
    if len(crop) == 0 or len(visible_model) == 0:
        return 0.0
    drawn = crop[draw(len(crop), budget, generator)]
    distances, _ = cKDTree(visible_model).query(drawn, k=1)
    return float((distances <= HEALTH_GATE).mean())


def rotation_error_degrees(first: FloatArray, second: FloatArray) -> float:
    cosine = (np.trace(first.T @ second) - 1.0) / 2.0
    return float(np.degrees(np.arccos(np.clip(cosine, -1.0, 1.0))))
