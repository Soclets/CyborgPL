#include <metal_stdlib>
using namespace metal;
kernel void touch(device uchar *b [[buffer(0)]], uint id [[thread_position_in_grid]]) { b[id] = b[id] + 1; }
