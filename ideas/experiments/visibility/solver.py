import numpy as np
from scipy.spatial import cKDTree
from scipy.spatial.transform import Rotation

from camera import FOCAL, IMAGE_HEIGHT, IMAGE_WIDTH, model_depth
from meshes import FloatArray, Pose

ITERATIONS = 12
WIDE_GATE = 0.03
NARROW_GATE = 0.012
DAMPING = 1e-4
STEP_LIMIT = 0.15
THROUGH_WEIGHT = 0.5
MINIMUM_PAIRS = 30


def lift(depth: FloatArray, rows: FloatArray, columns: FloatArray) -> FloatArray:
    return np.stack(
        [
            (columns + 0.5 - IMAGE_WIDTH / 2.0) * depth / FOCAL,
            (rows + 0.5 - IMAGE_HEIGHT / 2.0) * depth / FOCAL,
            depth,
        ],
        axis=-1,
    )


def normal_map(rendered: FloatArray) -> FloatArray:
    rows, columns = np.mgrid[0:IMAGE_HEIGHT, 0:IMAGE_WIDTH]
    points = lift(rendered, rows, columns)
    across = np.roll(points, -1, 1) - np.roll(points, 1, 1)
    down = np.roll(points, -1, 0) - np.roll(points, 1, 0)
    normals = np.cross(across, down)
    length = np.linalg.norm(normals, axis=-1, keepdims=True)
    normals = normals / np.where(length > 0.0, length, 1.0)
    return normals * np.where((normals * points).sum(axis=-1, keepdims=True) > 0.0, -1.0, 1.0)


def tukey(residual: FloatArray, gate: float) -> FloatArray:
    return np.where(np.abs(residual) < gate, (1.0 - (residual / gate) ** 2) ** 2, 0.0)


def plane_step(
    pose: Pose, model: FloatArray, scene: FloatArray, normals: FloatArray, weights: FloatArray
) -> Pose:
    centre = pose.translation
    arm = model - centre
    jacobian = np.concatenate([np.cross(arm, normals), normals], axis=1)
    residual = ((scene - model) * normals).sum(axis=1)
    weighted = jacobian * weights[:, None]
    hessian = weighted.T @ jacobian
    hessian += np.eye(6) * DAMPING * max(float(np.trace(hessian)) / 6.0, 1e-9)
    step = np.linalg.solve(hessian, weighted.T @ residual)
    turn = step[:3]
    size = np.linalg.norm(turn)
    if size > STEP_LIMIT:
        turn = turn * STEP_LIMIT / size
    rotation = Rotation.from_rotvec(turn).as_matrix()
    return Pose(rotation @ pose.rotation, pose.translation + step[3:])


def read_frame(depth: FloatArray, placed: FloatArray) -> FloatArray:
    depth_ahead = np.maximum(placed[:, 2], 1e-6)
    columns = np.floor(FOCAL * placed[:, 0] / depth_ahead + IMAGE_WIDTH / 2.0).astype(int)
    rows = np.floor(FOCAL * placed[:, 1] / depth_ahead + IMAGE_HEIGHT / 2.0).astype(int)
    inside = (
        (columns >= 0) & (columns < IMAGE_WIDTH) & (rows >= 0) & (rows < IMAGE_HEIGHT)
        & (placed[:, 2] > 1e-3)
    )  # fmt: skip
    seen = np.full(len(placed), np.nan)
    seen[inside] = depth[rows[inside], columns[inside]]
    return seen


def draw(count: int, budget: int, generator: np.random.Generator) -> FloatArray:
    return generator.choice(count, min(budget, count), replace=False)


def gate_at(iteration: int) -> float:
    return WIDE_GATE if iteration < ITERATIONS // 2 else NARROW_GATE


def one_way(
    start: Pose, cloud: FloatArray, points: FloatArray, normals: FloatArray, reach: float
) -> Pose:
    pose = start
    for iteration in range(ITERATIONS):
        gate = gate_at(iteration)
        placed, turned = pose.apply(points), pose.turn(normals)
        facing = (placed * turned).sum(axis=1) < 0.0
        crop = cloud[np.linalg.norm(cloud - pose.translation, axis=1) <= reach]
        if facing.sum() < MINIMUM_PAIRS or len(crop) < MINIMUM_PAIRS:
            return pose
        distance, nearest = cKDTree(placed[facing]).query(crop, k=1)
        model = placed[facing][nearest]
        planes = turned[facing][nearest]
        residual = ((crop - model) * planes).sum(axis=1)
        weights = tukey(residual, gate) * (distance <= 2.0 * gate)
        if (weights > 0.0).sum() < MINIMUM_PAIRS:
            return pose
        pose = plane_step(pose, model, crop, planes, weights)
    return pose


def along_rays(start: Pose, depth: FloatArray, mesh: FloatArray, through: bool = True) -> Pose:
    pose = start
    for iteration in range(ITERATIONS):
        gate = gate_at(iteration)
        rendered = model_depth(mesh, pose)
        planes = normal_map(rendered)
        rows, columns = np.nonzero(
            np.isfinite(rendered) & np.isfinite(depth) & np.isfinite(planes).all(axis=-1)
        )
        if len(rows) < MINIMUM_PAIRS:
            return pose
        expected, seen = rendered[rows, columns], depth[rows, columns]
        gap = seen - expected
        hit = np.abs(gap) < gate
        beyond = gap >= gate
        weights = tukey(gap, gate) + (THROUGH_WEIGHT * beyond if through else 0.0)
        if (weights > 0.0).sum() < MINIMUM_PAIRS:
            return pose
        target = expected + np.where(beyond, gate, gap) * (hit | beyond)
        pose = plane_step(
            pose,
            lift(expected, rows, columns),
            lift(target, rows, columns),
            planes[rows, columns],
            weights,
        )
    return pose


def one_way_budget(
    start: Pose,
    cloud: FloatArray,
    points: FloatArray,
    normals: FloatArray,
    reach: float,
    budget: int,
    generator: np.random.Generator,
) -> Pose:
    pose = start
    for iteration in range(ITERATIONS):
        gate = gate_at(iteration)
        placed, turned = pose.apply(points), pose.turn(normals)
        facing = (placed * turned).sum(axis=1) < 0.0
        crop = cloud[np.linalg.norm(cloud - pose.translation, axis=1) <= reach]
        if facing.sum() < 6 or len(crop) < 6:
            return pose
        drawn = crop[draw(len(crop), budget, generator)]
        distance, nearest = cKDTree(placed[facing]).query(drawn, k=1)
        model, planes = placed[facing][nearest], turned[facing][nearest]
        weights = tukey(((drawn - model) * planes).sum(axis=1), gate) * (distance <= 2.0 * gate)
        if (weights > 0.0).sum() < 6:
            return pose
        pose = plane_step(pose, model, drawn, planes, weights)
    return pose


def rays_budget(
    start: Pose,
    depth: FloatArray,
    points: FloatArray,
    normals: FloatArray,
    budget: int,
    generator: np.random.Generator,
    through: bool = False,
) -> Pose:
    pose = start
    for iteration in range(ITERATIONS):
        gate = gate_at(iteration)
        placed, turned = pose.apply(points), pose.turn(normals)
        facing = np.flatnonzero((placed * turned).sum(axis=1) < 0.0)
        if len(facing) < 6:
            return pose
        drawn = facing[draw(len(facing), budget, generator)]
        model, planes = placed[drawn], turned[drawn]
        gap = np.nan_to_num(read_frame(depth, model) - model[:, 2], nan=-np.inf)
        beyond = gap >= gate
        weights = tukey(gap, gate) + (THROUGH_WEIGHT * beyond if through else 0.0)
        if (weights > 0.0).sum() < 6:
            return pose
        moved = np.where(beyond, gate, np.where(np.abs(gap) < gate, gap, 0.0))
        scene = model * ((model[:, 2] + moved) / model[:, 2])[:, None]
        pose = plane_step(pose, model, scene, planes, weights)
    return pose
