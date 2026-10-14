///////////////////////////////////
// Entry point: "compute" (comp) //
///////////////////////////////////
#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
#extension GL_EXT_shader_atomic_int64 : require
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

struct _6
{
    uint _m0;
    uint64_t _m1;
    uint _m2;
};

layout(set = 0, binding = 0, std430) buffer _14_13
{
    _6 _m0[];
} _13;

layout(set = 0, binding = 1, std430) buffer _17_16
{
    uint64_t _m0[4];
} _16;

uvec2 _21(uint _20)
{
    return uvec2(4294967295u - _20, _20);
}

void main()
{
    if (gl_GlobalInvocationID.x < uint(_13._m0.length()))
    {
        uint64_t _47 = atomicMin(_13._m0[gl_GlobalInvocationID.x]._m1, packUint2x32(_21(gl_GlobalInvocationID.x)));
    }
    if (gl_GlobalInvocationID.x < 4u)
    {
        uint64_t _60 = atomicMax(_16._m0[gl_GlobalInvocationID.x], packUint2x32(_21(gl_GlobalInvocationID.x)));
    }
    if (gl_GlobalInvocationID.x == 0u)
    {
        uint64_t _71 = atomicMin(_16._m0[0u], packUint2x32(uvec2(1u, 2u)));
        return;
    }
    else
    {
        return;
    }
}


////////////////////////////////////
// Entry point: "fragment" (frag) //
////////////////////////////////////
#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
#extension GL_EXT_shader_atomic_int64 : require

struct _6
{
    uint _m0;
    uint64_t _m1;
    uint _m2;
};

layout(set = 0, binding = 0, std430) buffer _14_13
{
    _6 _m0[];
} _13;

layout(set = 0, binding = 1, std430) buffer _17_16
{
    uint64_t _m0[4];
} _16;

uvec2 _21(uint _20)
{
    return uvec2(4294967295u - _20, _20);
}

void main()
{
    uint _85 = uint(clamp(gl_FragCoord.x, 0.0, 4294967040.0));
    if (_85 < uint(_13._m0.length()))
    {
        uint64_t _87 = atomicMax(_13._m0[_85]._m1, packUint2x32(_21(_85)));
    }
}
