enable atomic_vec2u_min_max;
requires atomic_vec2u_min_max;

alias AtomicValue = atomic<vec2<u32>>;
struct Entry {
    padding: u32,
    value: AtomicValue,
    tail: u32,
}
@group(0) @binding(0) var<storage, read_write> entries: array<Entry>;
@group(0) @binding(1) var<storage, read_write> values: array<atomic<vec2u>, 4>;

fn candidate(i: u32) -> vec2u {
    return vec2u(0xffffffffu - i, i);
}

@compute @workgroup_size(64)
fn compute(@builtin(global_invocation_id) id: vec3u) {
    atomicStoreMin(&entries[id.x].value, candidate(id.x));
    atomicStoreMax(&values[id.x], candidate(id.x));
    if id.x == 0u {
        atomicStoreMin(&values[0], vec2(1, 2));
    }
}

@fragment
fn fragment(@builtin(position) position: vec4f) {
    let i = u32(position.x);
    atomicStoreMax(&entries[i].value, candidate(i));
}
