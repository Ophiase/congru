from typing import Final, Protocol

import numpy as np
import pytest

import recalage
from geometry import (
    GRID_STEP,
    Points,
    asymmetric_surface,
    box_mesh,
    moved,
    rigid_motion,
    rotation_error,
    translation_error,
)

SMALL_MOTION: Final = rigid_motion((0.04, -0.03, 0.05), (0.3 * GRID_STEP, 0.0, 0.1))
LARGE_MOTION: Final = rigid_motion((1.2, -2.0, 0.7), (0.5, -0.2, 0.3))
EXACT: Final = 1e-6
MESH_SPACING: Final = 0.1
NEIGHBORS: Final = 8


class TargetFactory(Protocol):
    def __call__(self, points: Points) -> recalage.Target: ...


def with_estimated_normals(points: Points) -> recalage.Target:
    return recalage.Target(points, neighbors=NEIGHBORS)


def without_normals(points: Points) -> recalage.Target:
    return recalage.Target(points)


@pytest.mark.parametrize(
    ("make_target", "icp"),
    [
        (with_estimated_normals, recalage.Icp()),
        (without_normals, recalage.Icp(metric=recalage.Metric.POINT_TO_POINT)),
    ],
)
def test_icp_recovers_a_small_motion(
    make_target: TargetFactory, icp: recalage.Icp
) -> None:
    model = asymmetric_surface()
    target = make_target(moved(model, SMALL_MOTION))
    alignment = icp.align(model, target)
    assert alignment.converged
    assert rotation_error(alignment.transform, SMALL_MOTION) < EXACT
    assert translation_error(alignment.transform, SMALL_MOTION) < EXACT


def test_a_mesh_target_pins_every_motion_of_its_own_samples() -> None:
    mesh = box_mesh()
    target = recalage.Target.from_mesh(mesh.vertices, mesh.triangles, MESH_SPACING)
    source = moved(target.points, np.linalg.inv(SMALL_MOTION))
    alignment = recalage.Icp().align(source, target)
    assert translation_error(alignment.transform, SMALL_MOTION) < EXACT
    assert alignment.observability is not None
    assert alignment.observability.weak_motions == ()


def test_multistart_recovers_a_large_rotation() -> None:
    model = asymmetric_surface()
    target = with_estimated_normals(moved(model, LARGE_MOTION))
    alignment = recalage.Multistart().align(model, target)
    assert rotation_error(alignment.transform, LARGE_MOTION) < EXACT


def test_point_to_plane_without_normals_is_refused() -> None:
    model = asymmetric_surface()
    with pytest.raises(recalage.RegistrationError, match="normals"):
        recalage.Icp().align(model, without_normals(model))
