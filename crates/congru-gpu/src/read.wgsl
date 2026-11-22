struct Grid {
    voxel: f32,
    width: u32,
    height: u32,
    depth: u32,
}

@group(0) @binding(0) var<uniform> grid: Grid;
@group(0) @binding(1) var<storage, read> values: array<f32>;
@group(0) @binding(2) var<storage, read> queries: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> answers: array<vec4<f32>>;

fn at(column: u32, row: u32, layer: u32) -> f32 {
    return values[column + grid.width * (row + grid.height * layer)];
}

@compute @workgroup_size(256)
fn read(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= arrayLength(&queries) {
        return;
    }
    let place = queries[index].xyz / grid.voxel;
    let floors = floor(place);
    let sizes = vec3<f32>(f32(grid.width), f32(grid.height), f32(grid.depth));
    if any(floors < vec3<f32>(0.0)) || any(floors >= sizes - vec3<f32>(1.0)) {
        answers[index] = vec4<f32>(0.0, 0.0, 0.0, 1e30);
        return;
    }
    let cell = vec3<u32>(floors);
    let share = place - floors;
    let c00 = at(cell.x, cell.y, cell.z);
    let c10 = at(cell.x + 1u, cell.y, cell.z);
    let c01 = at(cell.x, cell.y + 1u, cell.z);
    let c11 = at(cell.x + 1u, cell.y + 1u, cell.z);
    let d00 = at(cell.x, cell.y, cell.z + 1u);
    let d10 = at(cell.x + 1u, cell.y, cell.z + 1u);
    let d01 = at(cell.x, cell.y + 1u, cell.z + 1u);
    let d11 = at(cell.x + 1u, cell.y + 1u, cell.z + 1u);
    let e0 = mix(c00, c10, share.x);
    let e1 = mix(c01, c11, share.x);
    let f0 = mix(d00, d10, share.x);
    let f1 = mix(d01, d11, share.x);
    let g0 = mix(e0, e1, share.y);
    let g1 = mix(f0, f1, share.y);
    let near = mix(c10 - c00, c11 - c01, share.y);
    let far = mix(d10 - d00, d11 - d01, share.y);
    let gradient = vec3<f32>(mix(near, far, share.z), mix(e1 - e0, f1 - f0, share.z), g1 - g0);
    answers[index] = vec4<f32>(gradient / grid.voxel, mix(g0, g1, share.z));
}
