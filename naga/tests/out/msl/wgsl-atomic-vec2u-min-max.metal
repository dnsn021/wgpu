// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;
struct DefaultConstructible {
    template<typename T>
    operator T() && {
        return T {};
    }
};

struct _mslBufferSizes {
    uint size0;
};

typedef metal::atomic_ulong AtomicValue;
struct Entry {
    uint padding;
    char _pad1[4];
    AtomicValue value;
    uint tail;
    char _pad3[4];
};
typedef Entry type_2[1];
struct type_4 {
    metal::atomic_ulong inner[4];
};

metal::uint2 candidate(
    uint i
) {
    return metal::uint2(4294967295u - i, i);
}

struct computeInput {
};
[[max_total_threads_per_threadgroup(64)]] kernel void compute(
  metal::uint3 id [[thread_position_in_grid]]
, device type_2& entries [[user(fake0)]]
, device type_4& values [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint _e2 = id.x;
    metal::uint2 _e6 = candidate(id.x);
    const metal::uint2 atomic_value = _e6;
    uint(_e2) < 1 + (_buffer_sizes.size0 - 0 - 24) / 24 ? metal::atomic_min_explicit(&entries[_e2].value, (metal::ulong(atomic_value.x) | (metal::ulong(atomic_value.y) << 32)), metal::memory_order_relaxed) : (void)0;
    uint _e8 = id.x;
    metal::uint2 _e11 = candidate(id.x);
    const metal::uint2 atomic_value_1 = _e11;
    uint(_e8) < 4 ? metal::atomic_max_explicit(&values.inner[_e8], (metal::ulong(atomic_value_1.x) | (metal::ulong(atomic_value_1.y) << 32)), metal::memory_order_relaxed) : (void)0;
    if (id.x == 0u) {
        const metal::uint2 atomic_value_2 = metal::uint2(1u, 2u);
        metal::atomic_min_explicit(&values.inner[0], (metal::ulong(atomic_value_2.x) | (metal::ulong(atomic_value_2.y) << 32)), metal::memory_order_relaxed);
        return;
    } else {
        return;
    }
}

uint naga_f2u32(float value) {
    return static_cast<uint>(metal::clamp(value, 0.0, 4294967000.0));
}


struct fragment_Input {
};
fragment void fragment_(
  metal::float4 position [[position]]
, device type_2& entries [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint i_1 = naga_f2u32(position.x);
    metal::uint2 _e6 = candidate(i_1);
    const metal::uint2 atomic_value_3 = _e6;
    uint(i_1) < 1 + (_buffer_sizes.size0 - 0 - 24) / 24 ? metal::atomic_max_explicit(&entries[i_1].value, (metal::ulong(atomic_value_3.x) | (metal::ulong(atomic_value_3.y) << 32)), metal::memory_order_relaxed) : (void)0;
    return;
}
