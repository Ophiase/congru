use congru_embedded::{Error, KdTree, Real, Vector3};

use crate::fixtures::{cast, cube, rotation_floor};

fn agrees_with_brute_force<Float: Real>(count: usize, seed: u64) {
    let points: Vec<[Float; 3]> = cube(count, 1.0, [0.0; 3], seed)
        .into_iter()
        .map(cast)
        .collect();
    let tree = KdTree::new(&points).unwrap();
    for query in cube(200, 1.2, [0.1; 3], seed + 1) {
        let query = Vector3(cast::<Float>(query));
        let nearest = tree.nearest_within(query, Float::INFINITY).unwrap();
        let brute = points
            .iter()
            .map(|point| (Vector3(*point) - query).squared_norm())
            .fold(
                Float::INFINITY,
                |best, value| {
                    if value < best { value } else { best }
                },
            );
        assert_eq!(nearest.squared_distance, brute);
        assert_eq!(
            (tree.points()[nearest.index] - query).squared_norm(),
            nearest.squared_distance
        );
    }
}

#[test]
fn the_tree_finds_what_brute_force_finds_at_every_size() {
    for count in [1, 2, 8, 9, 17, 100, 1000, 4097] {
        agrees_with_brute_force::<f32>(count, 11);
        agrees_with_brute_force::<f64>(count, 12);
    }
}

fn duplicates_and_flat_data<Float: Real>() {
    let mut points = vec![[Float::ZERO; 3]; 64];
    for (index, point) in points.iter_mut().enumerate().take(32) {
        point[0] = Float::of_count(index);
    }
    let tree = KdTree::new(&points).unwrap();
    let query = Vector3([Float::of(31.2), Float::ZERO, Float::ZERO]);
    let nearest = tree.nearest_within(query, Float::INFINITY).unwrap();
    assert_eq!(tree.points()[nearest.index][0], Float::of(31.0));
}

#[test]
fn repeated_points_and_flat_clouds_are_searched_correctly() {
    duplicates_and_flat_data::<f32>();
    duplicates_and_flat_data::<f64>();
}

fn gate_holds<Float: Real>() {
    let tree = KdTree::new(&[[Float::ZERO; 3]]).unwrap();
    let query = Vector3([Float::ONE, Float::ZERO, Float::ZERO]);
    assert!(tree.nearest_within(query, Float::of(0.5)).is_none());
    assert!(tree.nearest_within(query, Float::of(1.5)).is_some());
}

#[test]
fn nothing_beyond_the_limit_is_returned() {
    gate_holds::<f32>();
    gate_holds::<f64>();
}

fn bad_input<Float: Real>() {
    assert_eq!(KdTree::<Float>::new(&[]).err(), Some(Error::Empty));
    let broken = [[Float::of(f64::NAN), Float::ZERO, Float::ZERO]];
    assert_eq!(KdTree::new(&broken).err(), Some(Error::NotFinite));
    let points = [[Float::ZERO; 3]; 2];
    assert!(KdTree::with_normals(&points, &points[..1]).is_err());
    let mut small = KdTree::new(&points).unwrap();
    assert!(small.estimate_normals::<3>().is_err());
}

#[test]
fn bad_input_is_refused() {
    bad_input::<f32>();
    bad_input::<f64>();
}

fn plane_normals<Float: Real>() {
    let tilt = [0.3, -0.4, 0.2];
    let points: Vec<[Float; 3]> = cube(500, 1.0, [0.0; 3], 13)
        .into_iter()
        .map(|[along, across, _]| {
            cast([along, across, tilt[0] * along + tilt[1] * across + tilt[2]])
        })
        .collect();
    let mut tree = KdTree::new(&points).unwrap();
    tree.estimate_normals::<8>().unwrap();
    let truth = Vector3(cast::<Float>([-tilt[0], -tilt[1], 1.0]));
    let truth = truth * (Float::ONE / truth.norm());
    for normal in tree.normals().unwrap() {
        let sine = normal.cross(truth).norm().as_f64();
        // Rounding a patch of spacing about 0.1 tilts it by ε over that lever.
        assert!(sine < rotation_floor::<Float>(1.0, 0.1), "{sine}");
    }
}

#[test]
fn a_plane_gives_its_own_normal() {
    plane_normals::<f32>();
    plane_normals::<f64>();
}
