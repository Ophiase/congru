use point_cloud::{ generation, linalg, PointCloud, Vec3 };

fn main() {
    println!("=== Point Cloud Library Demo ===\n");

    // Demo 1: Basic point cloud operations
    println!("1. Basic Point Cloud Operations:");
    let mut pc = PointCloud::new();
    pc.add_point(Vec3::new(0.0, 0.0, 0.0));
    pc.add_point(Vec3::new(1.0, 1.0, 1.0));
    pc.add_point(Vec3::new(-1.0, 2.0, 0.5));

    println!("   Point cloud has {} points", pc.len());
    println!("   Centroid: {:?}", pc.centroid());
    if let Some((min, max)) = pc.bounding_box() {
        println!("   Bounding box: min={:?}, max={:?}", min, max);
    }
    println!();

    // Demo 2: Grid generation
    println!("2. Grid Generation:");
    let grid_pc = generation::grid(
        0.5, (3, 3, 3)
    );
    println!("   Generated grid with {} points", grid_pc.len());
    println!("   First few points: {:?}", &grid_pc.points[..4]);
    println!();

    // Demo 3: Random box generation
    println!("3. Random Box Generation:");
    let box_pc = generation::random_box(Vec3::new(-2.0, -2.0, -2.0), Vec3::new(2.0, 2.0, 2.0), 20);
    println!("   Generated {} random points in box", box_pc.len());
    println!("   Sample points: {:?}", &box_pc.points[..3]);
    println!();

    // Demo 4: Sphere generation
    println!("4. Sphere Generation:");
    let sphere_surface = generation::sphere_surface(Vec3::new(0.0, 0.0, 0.0), 2.0, 15);
    let sphere_volume = generation::sphere_volume(Vec3::new(0.0, 0.0, 0.0), 2.0, 15);
    println!("   Generated {} points on sphere surface", sphere_surface.len());
    println!("   Generated {} points in sphere volume", sphere_volume.len());
    println!("   Surface sample: {:?}", &sphere_surface.points[0]);
    println!("   Volume sample: {:?}", &sphere_volume.points[0]);
    println!();

    // Demo 5: Iterative random walk
    println!("5. Iterative Random Walk:");
    let initial = vec![Vec3::new(0.0, 0.0, 0.0)];
    let walk_pc = generation::iterative_random_walk(initial, 5, 0.5, 3);
    println!("   Generated {} points through random walk", walk_pc.len());
    println!("   Final points spread: {:?}", walk_pc.bounding_box());
    println!();

    // Demo 6: Transformations
    println!("6. Rigid Transformations:");
    let mut transform_pc = generation::grid(1.0, (2, 2, 1));
    println!("   Original grid centroid: {:?}", transform_pc.centroid());

    let rotation = linalg::random_rotation();
    let translation = linalg::random_translation(3.0);

    transform_pc.transform(&rotation, &translation);
    println!("   After transformation centroid: {:?}", transform_pc.centroid());
    println!("   Translation applied: {:?}", translation);
    println!();

    // Demo 7: Merging point clouds
    println!("7. Merging Point Clouds:");
    let pc1 = generation::sphere_surface(Vec3::new(0.0, 0.0, 0.0), 1.0, 10);
    let pc2 = generation::sphere_surface(Vec3::new(3.0, 0.0, 0.0), 1.0, 10);
    let pc3 = generation::sphere_surface(Vec3::new(0.0, 3.0, 0.0), 1.0, 10);

    let merged = PointCloud::merge_multiple(&[pc1, pc2, pc3]);
    println!("   Merged {} point clouds into {} total points", 3, merged.len());
    println!("   Merged centroid: {:?}", merged.centroid());
    println!();

    println!("=== Demo Complete ===");
}
