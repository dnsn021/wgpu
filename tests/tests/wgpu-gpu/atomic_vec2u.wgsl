enable atomic_vec2u_min_max;

struct Entry {
    padding: u32,
    value: atomic<vec2u>,
    tail: u32,
}
@group(0) @binding(0) var<storage, read_write> output: array<Entry>;

struct ReadEntry {
    padding: u32,
    value: vec2u,
    tail: u32,
}
@group(0) @binding(1) var<storage, read> input: array<ReadEntry>;
@group(0) @binding(2) var<storage, read_write> values: array<vec2u>;

@compute @workgroup_size(4)
fn read_values(@builtin(global_invocation_id) id: vec3u) {
    values[id.x] = input[id.x].value;
}

fn update(i: u32) {
    let candidate = vec2u(511u - i, i);
    atomicStoreMin(&output[0].value, candidate);
    atomicStoreMax(&output[1].value, candidate);
    let unsigned = select(vec2u(0xffffffffu, 0x7fffffffu), vec2u(0u, 0x80000000u), i == 0u);
    atomicStoreMin(&output[2].value, unsigned);
    atomicStoreMax(&output[3].value, unsigned);
}

@compute @workgroup_size(64)
fn compute(@builtin(global_invocation_id) id: vec3u) {
    update(id.x);
}

@vertex
fn vertex(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let positions = array(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
    return vec4f(positions[i], 0, 1);
}

@fragment
fn fragment(@builtin(position) position: vec4f) -> @location(0) vec4f {
    update(u32(position.y) * 32u + u32(position.x));
    return vec4f(0);
}
