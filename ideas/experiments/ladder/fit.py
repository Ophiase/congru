from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray
from scipy.spatial import cKDTree

from scenes import BRACKET, Points, depths_along_rays, outline_samples, placed

ITERATIONS = 40
HIT_GATE = 0.012
START_TURNS = np.radians([-30.0, -15.0, 15.0, 30.0])
START_SHIFTS = ((0.03, 0.02), (-0.03, -0.02))


@dataclass(frozen=True)
class Pose:
    angle: float
    shift: Points


@dataclass(frozen=True)
class Verdict:
    hits: int
    through: int
    hidden: int


def rigid_fit(moving: Points, fixed: Points) -> Pose:
    moving_centre, fixed_centre = moving.mean(axis=0), fixed.mean(axis=0)
    across = (moving - moving_centre).T @ (fixed - fixed_centre)
    angle = float(np.arctan2(across[0, 1] - across[1, 0], across[0, 0] + across[1, 1]))
    turn = np.array([[np.cos(angle), -np.sin(angle)], [np.sin(angle), np.cos(angle)]])
    return Pose(angle, fixed_centre - turn @ moving_centre)


def scan_to_model(scan: Points, start: Pose) -> Pose:
    samples = outline_samples(BRACKET)
    pose = start
    for _ in range(ITERATIONS):
        _, nearest = cKDTree(placed(samples, pose.angle, pose.shift)).query(scan)
        pose = rigid_fit(samples[nearest], scan)
    return pose


def model_to_scan(scan: Points, start: Pose) -> Pose:
    samples = outline_samples(BRACKET)
    pose = start
    tree = cKDTree(scan)
    for _ in range(ITERATIONS):
        _, nearest = tree.query(placed(samples, pose.angle, pose.shift))
        pose = rigid_fit(samples, scan[nearest])
    return pose


def starts(truth: Pose) -> tuple[Pose, ...]:
    return tuple(
        Pose(truth.angle + turn, truth.shift + np.array(shift))
        for turn in START_TURNS
        for shift in START_SHIFTS
    )


def rotation_error(pose: Pose, truth: Pose) -> float:
    gap = np.arctan2(np.sin(pose.angle - truth.angle), np.cos(pose.angle - truth.angle))
    return float(np.degrees(abs(gap)))


def verdict(pose: Pose, observed: NDArray[np.float64]) -> Verdict:
    model = depths_along_rays((placed(BRACKET, pose.angle, pose.shift),))
    on_model = np.isfinite(model)
    gap = observed[on_model] - model[on_model]
    return Verdict(
        hits=int((np.abs(gap) <= HIT_GATE).sum()),
        through=int((gap > HIT_GATE).sum()),
        hidden=int((gap < -HIT_GATE).sum()),
    )
