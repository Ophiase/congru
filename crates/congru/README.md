# congru

Rigid registration of point clouds and meshes: the rotation and translation that
bring one view onto another, and which motions the data left free.

| You know | Use |
| --- | --- |
| Which point goes with which | `fit_rigid`, closed form |
| A pose within about 20° | `Icp`, point-to-point or point-to-plane |
| Nothing | `Multistart`, ICP from rotations spread over SO(3) |

```rust
use congru::{Icp, Point, PointCloud, RigidTransform, Target};

let model: Vec<Point> = (0..400)
    .map(|index| {
        let along = f64::from(index % 20) * 0.05;
        let across = f64::from(index / 20) * 0.05;
        Point::new(along, across, (3.0 * along).sin() * (2.0 * across).cos())
    })
    .collect();
let target = Target::with_estimated_normals(PointCloud::new(model.clone())?, 8)?;
let turn = nalgebra::Vector3::z() * 0.05;
let truth = RigidTransform::from_axis_angle(turn, [0.02, 0.0, 0.01]);
let source = PointCloud::new(model)?.transformed(&truth.inverse());

let alignment = Icp::default().align(&source, &target, &RigidTransform::identity())?;
assert!(alignment.transform.rotation_error_to(&truth) < 1e-6);
# Ok::<(), congru::Error>(())
```

It runs on the CPU, in parallel by default; `default-features = false` makes it
single-threaded. The API is not frozen at 0.1.

Licence: Apache 2.0. The project, its demos and its benchmark:
[github.com/Ophiase/congru](https://github.com/Ophiase/congru).
