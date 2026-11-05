"""Fit along the camera's rays: what a pixel says, not which point is nearest."""

from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray

from fit import HIT_GATE, Pose, rigid_fit
from scenes import BRACKET, CAMERA, RAY_COUNT, Points, depths_along_rays, placed, ray_directions

GATES = (0.06, 0.04, 0.025, 0.012)
PAIRING_ROUNDS = 6
WINDOW = 2
THROUGH_COST = 3.0
POLISH_ROUNDS = 60
POLISH_TURN = np.radians(2.0)
POLISH_SHIFT = 0.008
POLISH_SHRINK = 0.5
LEAST_PAIRS = 3


@dataclass(frozen=True)
class Reading:
    hits: int
    through: int
    hidden: int
    value: float


def seen_points(observed: NDArray[np.float64]) -> Points:
    return CAMERA + ray_directions() * observed[:, None]


def visible_model(pose: Pose) -> tuple[Points, NDArray[np.bool_]]:
    depths = depths_along_rays((placed(BRACKET, pose.angle, pose.shift),))
    on_model = np.isfinite(depths)
    points = CAMERA + ray_directions() * np.where(on_model, depths, 0.0)[:, None]
    return points, on_model


def read(pose: Pose, observed: NDArray[np.float64]) -> Reading:
    model = depths_along_rays((placed(BRACKET, pose.angle, pose.shift),))
    known = np.isfinite(model) & np.isfinite(observed)
    gap = observed[known] - model[known]
    hits = int((np.abs(gap) <= HIT_GATE).sum())
    through = int((gap > HIT_GATE).sum())
    hidden = int((gap < -HIT_GATE).sum())
    return Reading(hits, through, hidden, hits - THROUGH_COST * through)


def pairs(pose: Pose, observed: NDArray[np.float64], gate: float) -> tuple[Points, Points]:
    points, on_model = visible_model(pose)
    seen = seen_points(observed)
    usable = np.isfinite(observed)
    model_side, frame_side = [], []
    for ray in np.flatnonzero(on_model):
        low, high = max(0, ray - WINDOW), min(RAY_COUNT, ray + WINDOW + 1)
        candidates = seen[low:high][usable[low:high]]
        if len(candidates) == 0:
            continue
        distances = np.linalg.norm(candidates - points[ray], axis=1)
        nearest = int(distances.argmin())
        if distances[nearest] <= gate:
            model_side.append(points[ray])
            frame_side.append(candidates[nearest])
    return np.array(model_side).reshape(-1, 2), np.array(frame_side).reshape(-1, 2)


def compose(step: Pose, pose: Pose) -> Pose:
    turn = np.array([[np.cos(step.angle), -np.sin(step.angle)], [np.sin(step.angle), np.cos(step.angle)]])
    return Pose(pose.angle + step.angle, turn @ pose.shift + step.shift)


def along_rays(start: Pose, observed: NDArray[np.float64]) -> Pose:
    pose = start
    for gate in GATES:
        for _ in range(PAIRING_ROUNDS):
            model_side, frame_side = pairs(pose, observed, gate)
            if len(model_side) < LEAST_PAIRS:
                break
            pose = compose(rigid_fit(model_side, frame_side), pose)
    return pose


def polish(start: Pose, observed: NDArray[np.float64]) -> Pose:
    pose, best = start, read(start, observed).value
    turn, shift = POLISH_TURN, POLISH_SHIFT
    moves = ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1))
    for _ in range(POLISH_ROUNDS):
        better = False
        for spin, across, along in moves:
            trial = Pose(pose.angle + spin * turn, pose.shift + shift * np.array([across, along]))
            value = read(trial, observed).value
            if value > best:
                pose, best, better = trial, value, True
        if not better:
            turn, shift = turn * POLISH_SHRINK, shift * POLISH_SHRINK
    return pose


def by_rays(start: Pose, observed: NDArray[np.float64]) -> Pose:
    return polish(along_rays(start, observed), observed)
