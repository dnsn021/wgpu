enable atomic_vec2u_min_max;

struct Entry {
    padding: u32,
    value: atomic<vec2<u32>>,
    tail: u32,
}

@group(0) @binding(0)
var<storage, read_write> entries: array<Entry>;
@group(0) @binding(1)
var<storage, read_write> values: array<atomic<vec2<u32>>, 4>;

fn candidate(i: u32) -> vec2<u32> {
    return vec2<u32>((4294967295u - i), i);
}

@compute @workgroup_size(64, 1, 1)
fn compute(@builtin(global_invocation_id) id: vec3<u32>) {
    let _e6 = candidate(id.x);
    atomicStoreMin((&entries[id.x].value), _e6);
    let _e11 = candidate(id.x);
    atomicStoreMax((&values[id.x]), _e11);
    if (id.x == 0u) {
        atomicStoreMin((&values[0]), vec2<u32>(1u, 2u));
        return;
    } else {
        return;
    }
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) {
    let i_1 = u32(position.x);
    let _e6 = candidate(i_1);
    atomicStoreMax((&entries[i_1].value), _e6);
    return;
}
