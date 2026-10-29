from typing import Final, Protocol

import numpy as np
import pytest

import recalage
from geometry import Points, asymmetric_surface, moved, rigid_motion, rotation_error

MOTION: Final = rigid_motion((0.3, 0.2, -0.4), (1.0, -2.0, 0.5))
ROUNDING: Final = 1e-9


class Layout(Protocol):
    def __call__(self, array: Points) -> Points: ...


@pytest.mark.parametrize("layout", [np.ascontiguousarray, np.asfortranarray])
def test_fit_rigid_recovers_the_motion_whatever_the_layout(
    layout: Layout,
) -> None:
    model = asymmetric_surface()
    fit = recalage.fit_rigid(layout(model), layout(moved(model, MOTION)))
    assert rotation_error(fit.transform, MOTION) < ROUNDING
    assert np.allclose(fit.transform[:3, 3], MOTION[:3, 3], atol=ROUNDING)
    assert fit.rmse < ROUNDING


def test_a_list_of_rows_reads_as_an_array() -> None:
    model = asymmetric_surface()
    as_array = recalage.Target(model).points
    as_lists = recalage.Target(model.tolist()).points
    assert np.array_equal(as_array, as_lists)


@pytest.mark.parametrize(
    "points",
    [np.zeros((5, 2)), np.zeros(9), np.zeros((2, 3, 3))],
    ids=["two columns", "flat", "three axes"],
)
def test_points_must_be_rows_of_three(points: Points) -> None:
    with pytest.raises(ValueError, match=r"shape \(n, 3\)"):
        recalage.Target(points)


def test_an_initial_pose_must_be_four_by_four() -> None:
    model = asymmetric_surface()
    target = recalage.Target(model)
    icp = recalage.Icp(metric=recalage.Metric.POINT_TO_POINT)
    with pytest.raises(ValueError, match=r"shape \(4, 4\)"):
        icp.align(model, target, initial=np.eye(3))
