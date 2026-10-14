struct Entry {
    uint padding;
    int _pad1_0;
    uint64_t value;
    uint tail;
    int _end_pad_0;
};

RWByteAddressBuffer entries : register(u0);
RWByteAddressBuffer values : register(u1);

struct FragmentInput_fragment {
    float4 position_1 : SV_Position;
};

uint2 candidate(uint i)
{
    return uint2((4294967295u - i), i);
}

[numthreads(64, 1, 1)]
void compute(uint3 id : SV_DispatchThreadID)
{
    const uint2 _e6 = candidate(id.x);
    const uint2 atomic_value = _e6;
    const uint64_t atomic_value64_ = uint64_t(atomic_value.x) | (uint64_t(atomic_value.y) << 32);
    entries.InterlockedMin64(8+id.x*24, atomic_value64_);
    const uint2 _e11 = candidate(id.x);
    const uint2 atomic_value_1 = _e11;
    const uint64_t atomic_value64_1 = uint64_t(atomic_value_1.x) | (uint64_t(atomic_value_1.y) << 32);
    values.InterlockedMax64(id.x*8, atomic_value64_1);
    if ((id.x == 0u)) {
        const uint2 atomic_value_2 = uint2(1u, 2u);
        const uint64_t atomic_value64_2 = uint64_t(atomic_value_2.x) | (uint64_t(atomic_value_2.y) << 32);
        values.InterlockedMin64(0, atomic_value64_2);
        return;
    } else {
        return;
    }
}

uint naga_f2u32(float value) {
    return uint(clamp(value, 0.0, 4294967000.0));
}

void fragment(FragmentInput_fragment fragmentinput_fragment)
{
    float4 position = fragmentinput_fragment.position_1;
    uint i_1 = naga_f2u32(position.x);
    const uint2 _e6 = candidate(i_1);
    const uint2 atomic_value_3 = _e6;
    const uint64_t atomic_value64_3 = uint64_t(atomic_value_3.x) | (uint64_t(atomic_value_3.y) << 32);
    entries.InterlockedMax64(8+i_1*24, atomic_value64_3);
    return;
}
