import numpy as np

from meshes import (
    FloatArray,
    Pose,
    bracket,
    can_in_front,
    room,
    turntable,
    turntable_pose,
)

IMAGE_WIDTH = 160
IMAGE_HEIGHT = 120
FOCAL = IMAGE_WIDTH / 2.0 / np.tan(np.radians(30.0))
NOISE_AT_ONE_METRE = 0.025
HOLE_SHARE = 0.08
HOLE_RADIUS = 2
EDGE_JUMP = 0.02
TRANSLATION_NUDGE = 0.003


def render(triangles: FloatArray) -> FloatArray:
    depth = np.full((IMAGE_HEIGHT, IMAGE_WIDTH), np.inf)
    columns = FOCAL * triangles[:, :, 0] / np.maximum(triangles[:, :, 2], 1e-6) + IMAGE_WIDTH / 2.0
    rows = FOCAL * triangles[:, :, 1] / np.maximum(triangles[:, :, 2], 1e-6) + IMAGE_HEIGHT / 2.0
    inverse = 1.0 / np.maximum(triangles[:, :, 2], 1e-6)
    in_front = (triangles[:, :, 2] > 1e-3).all(axis=1)
    for index in np.flatnonzero(in_front):
        across, down = columns[index], rows[index]
        left = max(int(np.floor(across.min())), 0)
        right = min(int(np.ceil(across.max())), IMAGE_WIDTH - 1)
        top = max(int(np.floor(down.min())), 0)
        bottom = min(int(np.ceil(down.max())), IMAGE_HEIGHT - 1)
        if left > right or top > bottom:
            continue
        grid_rows, grid_columns = np.mgrid[top : bottom + 1, left : right + 1]
        centre_x, centre_y = grid_columns + 0.5, grid_rows + 0.5
        denominator = (down[1] - down[2]) * (across[0] - across[2]) + (across[2] - across[1]) * (
            down[0] - down[2]
        )
        if abs(denominator) < 1e-12:
            continue
        first = (
            (down[1] - down[2]) * (centre_x - across[2])
            + (across[2] - across[1]) * (centre_y - down[2])
        ) / denominator
        second = (
            (down[2] - down[0]) * (centre_x - across[2])
            + (across[0] - across[2]) * (centre_y - down[2])
        ) / denominator
        third = 1.0 - first - second
        inside = (first >= 0.0) & (second >= 0.0) & (third >= 0.0)
        if not inside.any():
            continue
        values = 1.0 / (
            first * inverse[index, 0] + second * inverse[index, 1] + third * inverse[index, 2]
        )
        window = depth[top : bottom + 1, left : right + 1]
        window[inside] = np.minimum(window[inside], values[inside])
    depth[~np.isfinite(depth)] = np.nan
    return depth


def transformed(triangles: FloatArray, pose: Pose) -> FloatArray:
    return triangles @ pose.rotation.T + pose.translation


def compose_depth(*layers: FloatArray) -> FloatArray:
    stacked = np.stack([np.nan_to_num(layer, nan=np.inf) for layer in layers])
    closest = stacked.min(axis=0)
    closest[~np.isfinite(closest)] = np.nan
    return closest


def degrade(exact: FloatArray, generator: np.random.Generator) -> FloatArray:
    finite = np.isfinite(exact)
    padded = np.pad(np.where(finite, exact, 0.0), 1)
    weights = np.pad(finite.astype(float), 1)
    sums = sum(
        np.roll(np.roll(padded, row, 0), column, 1) for row in (-1, 0, 1) for column in (-1, 0, 1)
    )
    counts = sum(
        np.roll(np.roll(weights, row, 0), column, 1) for row in (-1, 0, 1) for column in (-1, 0, 1)
    )
    blurred = np.where(finite, sums[1:-1, 1:-1] / np.maximum(counts[1:-1, 1:-1], 1.0), np.nan)
    farthest = np.nanmax(
        np.stack(
            [
                np.roll(np.roll(blurred, row, 0), column, 1)
                for row in (-1, 0, 1)
                for column in (-1, 0, 1)
            ]
        ),
        axis=0,
    )
    flying = np.isfinite(blurred) & (farthest - blurred > EDGE_JUMP)
    mixed = np.where(flying, (blurred + farthest) / 2.0, blurred)
    noisy = mixed + generator.normal(0.0, NOISE_AT_ONE_METRE, mixed.shape) * mixed * mixed
    disc_area = np.pi * HOLE_RADIUS**2
    for _ in range(round(HOLE_SHARE * IMAGE_WIDTH * IMAGE_HEIGHT / disc_area)):
        row, column = generator.integers(IMAGE_HEIGHT), generator.integers(IMAGE_WIDTH)
        row_grid, column_grid = np.ogrid[: IMAGE_HEIGHT, : IMAGE_WIDTH]
        noisy[(row_grid - row) ** 2 + (column_grid - column) ** 2 <= HOLE_RADIUS**2] = np.nan
    return noisy


def back_project(depth: FloatArray) -> FloatArray:
    rows, columns = np.nonzero(np.isfinite(depth))
    distance = depth[rows, columns]
    return np.stack(
        [
            (columns + 0.5 - IMAGE_WIDTH / 2.0) * distance / FOCAL,
            (rows + 0.5 - IMAGE_HEIGHT / 2.0) * distance / FOCAL,
            distance,
        ],
        axis=1,
    )


def sample_surface(
    triangles: FloatArray, count: int, generator: np.random.Generator
) -> tuple[FloatArray, FloatArray]:
    edges_one = triangles[:, 1] - triangles[:, 0]
    edges_two = triangles[:, 2] - triangles[:, 0]
    crossed = np.cross(edges_one, edges_two)
    areas = np.linalg.norm(crossed, axis=1)
    chosen = generator.choice(len(triangles), count, p=areas / areas.sum())
    first, second = generator.random(count), generator.random(count)
    flip = first + second > 1.0
    first[flip], second[flip] = 1.0 - first[flip], 1.0 - second[flip]
    points = (
        triangles[chosen, 0]
        + edges_one[chosen] * first[:, None]
        + edges_two[chosen] * second[:, None]
    )
    normals = crossed[chosen] / areas[chosen, None]
    return points, normals


def model_depth(mesh: FloatArray, pose: Pose) -> FloatArray:
    return render(transformed(mesh, pose))


def frame_depth(spin: float, seed: int, with_can: bool) -> tuple[FloatArray, Pose]:
    generator = np.random.default_rng(seed)
    truth = turntable_pose(spin, sway=spin)
    layers = [
        model_depth(bracket(), truth),
        render(transformed(turntable(), truth)),
        render(room()),
    ]
    if with_can:
        layers.append(render(can_in_front(0.02)))
    return degrade(compose_depth(*layers), generator), truth


def turn_about(axis: FloatArray, radians: float) -> FloatArray:
    cross = np.array(
        [[0.0, -axis[2], axis[1]], [axis[2], 0.0, -axis[0]], [-axis[1], axis[0], 0.0]]
    )
    return np.eye(3) + np.sin(radians) * cross + (1.0 - np.cos(radians)) * cross @ cross


def nudge(truth: Pose, degrees: float, generator: np.random.Generator) -> Pose:
    axis = generator.normal(size=3)
    axis /= np.linalg.norm(axis)
    return Pose(
        turn_about(axis, np.radians(degrees)) @ truth.rotation,
        truth.translation + generator.normal(0.0, TRANSLATION_NUDGE, 3),
    )
