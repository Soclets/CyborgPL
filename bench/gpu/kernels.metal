#include <metal_stdlib>
using namespace metal;

kernel void sum(device ulong *part [[buffer(0)]], constant long &n [[buffer(1)]],
                uint t [[thread_position_in_grid]], uint nt [[threads_per_grid]]) {
    ulong acc = 0;
    for (long i = 1 + t; i <= n; i += nt) acc += (ulong)((i * i) % 7);
    part[t] = acc;
}

kernel void map(device int *xs [[buffer(0)]], uint i [[thread_position_in_grid]]) {
    xs[i] = xs[i] * 3 + 1;
}

kernel void mandel(device uint *rows [[buffer(0)]], constant uint &w [[buffer(1)]],
                   uint2 p [[thread_position_in_grid]]) {
    float cr = -2.0f + 3.0f * p.x / w, ci = -1.5f + 3.0f * p.y / w;
    float zr = 0, zi = 0; int k = 0;
    for (; k < 256 && zr * zr + zi * zi <= 4.0f; k++) { float t = zr * zr - zi * zi + cr; zi = 2 * zr * zi + ci; zr = t; }
    if (k == 256) atomic_fetch_add_explicit((device atomic_uint *)&rows[p.y], 1, memory_order_relaxed);
}
