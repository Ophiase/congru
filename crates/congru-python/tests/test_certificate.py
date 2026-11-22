from typing import Final

import numpy as np

import congru
from geometry import asymmetric_surface, box_mesh, moved, rigid_motion, rotation_error

MOTION: Final = rigid_motion((0.4, -0.3, 0.6), (0.1, 0.0, -0.05))
MARKS: Final = 40
SCALE: Final = 1.07
EXACT: Final = 1e-6
NEAR_TRUTH: Final = 0.1
ASKED_GAP: Final = 1e-3
WRONG_VALUE: Final = 0.03


def marks() -> np.ndarray:
    surface = asymmetric_surface()
    return surface[:: len(surface) // MARKS][:MARKS]


def test_the_truth_is_among_the_survivors() -> None:
    target = asymmetric_surface()
    certificate = congru.Certifier(
        truncation=0.2,
        reach=1.5,
        rotation_resolution=0.05,
        translation_resolution=0.02,
    ).run(moved(marks(), np.linalg.inv(MOTION)), target)
    assert certificate.closed
    nearest = min(rotation_error(pose, MOTION) for pose in certificate.survivors)
    assert nearest < NEAR_TRUTH


def test_a_value_past_the_optimum_shows_a_gap() -> None:
    target = asymmetric_surface()
    source = moved(marks(), np.linalg.inv(MOTION))
    wrong = rigid_motion((0.0, 2.5, 0.0), (0.0, 0.0, 0.0))
    certificate = congru.Certifier(
        truncation=0.2,
        reach=1.5,
        margin=-ASKED_GAP,
        rotation_resolution=0.05,
        translation_resolution=0.02,
    ).run(source, target, [wrong])
    assert certificate.upper < ASKED_GAP
    assert certificate.gap(WRONG_VALUE) > WRONG_VALUE - 2 * ASKED_GAP


def test_a_motion_is_recovered_on_a_mesh_and_its_field() -> None:
    mesh = box_mesh()
    small = rigid_motion((0.03, -0.02, 0.04), (0.02, 0.0, 0.01))
    samples = congru.Target.from_mesh(mesh.vertices, mesh.triangles, 0.1).points
    scan = moved(samples, np.linalg.inv(small))
    for surface in (
        congru.Surface.exact(mesh.vertices, mesh.triangles),
        congru.Surface.field(mesh.vertices, mesh.triangles, 0.02, 0.3),
    ):
        found = surface.align(scan, congru.Icp())
        assert rotation_error(found.transform, small) < NEAR_TRUTH


def test_a_similarity_recovers_a_known_scale() -> None:
    source = marks()
    target = moved(SCALE * source, MOTION)
    fit = congru.fit_similarity(source, target)
    assert abs(fit.scale - SCALE) < EXACT
    assert rotation_error(fit.transform, MOTION) < EXACT
